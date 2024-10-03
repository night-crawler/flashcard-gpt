use crate::chat_manager::ChatManager;
use crate::ext::StrExt;
use crate::patch_state;
use crate::schema::receive_next;
use crate::schema::root::{cancel, handle_show_generic_menu};
use anyhow::anyhow;
use flashcard_gpt_core::dto::card::CreateCardDto;
use flashcard_gpt_core::dto::deck_card::CreateDeckCardDto;
use flashcard_gpt_core::dto::llm::GptCardGroup;
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Update};
use tracing::{error, info};
use crate::command::card::CardCommand;
use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;

pub fn card_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let card_command_handler = teloxide::filter_command::<CardCommand, _>().branch(
        case![BotState::InsideCardMenu(fields)]
            .branch(case![CardCommand::Create].endpoint(handle_create_card))
            .branch(case![CardCommand::Generate].endpoint(handle_generate_cards)),
    );

    let card_message_handler = Update::filter_message()
        .branch(card_command_handler)
        .branch(
            teloxide::filter_command::<CardCommand, _>()
                .branch(case![CardCommand::Cancel].endpoint(cancel)),
        )
        .branch(case![BotState::ReceiveCardTitle(fields)].endpoint(receive_card_title))
        .branch(case![BotState::ReceiveCardFront(fields)].endpoint(receive_card_front))
        .branch(case![BotState::ReceiveCardBack(fields)].endpoint(receive_card_back))
        .branch(case![BotState::ReceiveCardHints(fields)].endpoint(receive_card_hints))
        .branch(case![BotState::ReceiveCardDifficulty(fields)].endpoint(receive_card_difficulty))
        .branch(case![BotState::ReceiveCardImportance(fields)].endpoint(receive_card_importance))
        .branch(
            case![BotState::ReceiveCardTags(fields)]
                .branch(
                    teloxide::filter_command::<CardCommand, _>()
                        .branch(case![CardCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_card_tags),
        )
        .branch(
            case![BotState::ReceiveCardDeck(fields)]
                .branch(
                    teloxide::filter_command::<CardCommand, _>()
                        .branch(case![CardCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_card_deck),
        )
        .branch(
            case![BotState::ReceiveCardConfirm(fields)].branch(
                teloxide::filter_command::<CardCommand, _>()
                    .branch(case![CardCommand::Next].endpoint(create_card)),
            ),
        )
        .branch(case![BotState::ReceiveGenerateCardPrompt(fields)].endpoint(receive_generator_prompt))
        .branch(
            case![BotState::ReceiveGenerateCardConfirm(fields)].branch(
                teloxide::filter_command::<CardCommand, _>()
                    .branch(case![CardCommand::Next].endpoint(generate_cards)),
            ),
        );

    card_message_handler
}

pub async fn handle_create_card(manager: ChatManager) -> anyhow::Result<()> {
    manager
        .send_message(
            "You are creating a new card.\nUse /cancel to exit and /next to skip the step.\n",
        )
        .await?;
    manager
        .update_state(BotState::ReceiveCardTitle(StateFields::default_card()))
        .await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_card_title(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_title) = manager.parse_html() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };
    let fields = patch_state!(
        manager,
        StateFields::Card { title },
        |title: &mut Option<Arc<str>>| { title.replace(next_title) }
    );

    manager
        .update_state(BotState::ReceiveCardFront(fields))
        .await?;
    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn receive_card_front(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_front) = manager.parse_html() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Card { front },
        |front: &mut Option<Arc<str>>| { front.replace(next_front) }
    );
    manager.update_state(BotState::ReceiveCardBack(fields)).await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_card_back(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_back) = manager.parse_html() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(manager, StateFields::Card { back }, |back: &mut Option<
        Arc<str>,
    >| {
        back.replace(next_back)
    });

    manager
        .update_state(BotState::ReceiveCardHints(fields))
        .await?;
    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn receive_card_hints(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_hints) = manager.parse_html_values("\n\n") else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(manager, StateFields::Card { hints }, |hints: &mut Vec<
        Arc<str>,
    >| {
        hints.extend(next_hints)
    });
    manager
        .update_state(BotState::ReceiveCardDifficulty(fields))
        .await?;

    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_card_difficulty(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_difficulty) = manager.parse_integer::<u8>() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };
    let fields = patch_state!(
        manager,
        StateFields::Card { difficulty },
        |difficulty: &mut Option<u8>| { difficulty.replace(next_difficulty) }
    );

    manager
        .update_state(BotState::ReceiveCardImportance(fields))
        .await?;

    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn receive_card_importance(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_importance) = manager.parse_integer::<u8>() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Card { importance },
        |importance: &mut Option<u8>| { importance.replace(next_importance) }
    );

    manager.update_state(BotState::ReceiveCardTags(fields)).await?;
    manager.send_tag_menu().await?;
    Ok(())
}

async fn receive_card_tags(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_tags) = manager.parse_html_values(',') else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Card { tags },
        |tags: &mut BTreeSet<Arc<str>>| { tags.extend(next_tags) }
    );
    manager.update_state(BotState::ReceiveCardTags(fields)).await?;
    manager.send_tag_menu().await?;

    Ok(())
}

async fn receive_card_deck(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_invalid_input().await?;
    Ok(())
}

async fn create_card(manager: ChatManager) -> anyhow::Result<()> {
    let StateFields::Card {
        id: _id,
        title,
        front,
        back,
        hints,
        difficulty,
        importance,
        data,
        tags,
        deck,
    } = manager.get_state().await?.into_fields()
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let user = &manager.binding.user;

    let tags = manager
        .repositories
        .tags
        .get_or_create_tags(user.as_ref(), tags)
        .await?
        .into_iter()
        .map(|tag| tag.id)
        .collect();

    let title = title.ok_or_else(|| anyhow!("Title was not provided"))?;

    let card = manager
        .repositories
        .cards
        .create(CreateCardDto {
            user: user.id.clone(),
            title,
            front,
            back,
            hints,
            difficulty: difficulty.unwrap_or(0),
            importance: importance.unwrap_or(0),
            data,
            tags,
        })
        .await?;

    manager
        .send_message(format!("Created a new card: {card:?}"))
        .await?;

    if let Some(deck) = deck {
        let rel = manager
            .repositories
            .decks
            .relate_card(CreateDeckCardDto {
                deck: deck.as_thing()?,
                card: card.id.clone(),
            })
            .await?;
        manager
            .send_message(format!("Related card to deck: {rel:?}"))
            .await?;
    }

    manager.dialogue.exit().await?;
    Ok(())
}

pub async fn handle_generate_cards(manager: ChatManager) -> anyhow::Result<()> {
    manager
        .update_state(BotState::ReceiveGenerateCardDeck(StateFields::GenerateCard {
            deck: None,
            prompt: None,
        }))
        .await?;
    manager.send_deck_menu().await?;
    Ok(())
}

async fn receive_generator_prompt(manager: ChatManager) -> anyhow::Result<()> {
    let Some(text) = manager.parse_html() else {
        manager.send_invalid_input().await?;
        error!("Prompt was not provided.");
        return Ok(());
    };
    info!(%text, "Received a prompt for card generation");

    let fields = patch_state!(
        manager,
        StateFields::GenerateCard { prompt },
        |prompt: &mut Option<Arc<str>>| { prompt.replace(text) }
    );

    manager
        .update_state(BotState::ReceiveGenerateCardConfirm(fields))
        .await?;
    manager.send_state_and_prompt().await?;
    manager.send_menu::<CardCommand>().await?;

    Ok(())
}

async fn generate_cards(
    manager: ChatManager,
    generator: CardGeneratorService,
) -> anyhow::Result<()> {
    let StateFields::GenerateCard {
        deck: Some(deck),
        prompt: Some(prompt),
    } = manager.get_state().await?.into_fields()
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let user = manager.binding.user.clone();

    let (code_cards, params) = generator.generate_code_cards(prompt.as_ref()).await?;
    let mut gpt_card_group = GptCardGroup::from_gpt_response(&code_cards)?;

    if let Some(data) = gpt_card_group.data.as_mut()
        && let Value::Object(map) = data
    {
        map.insert("prompt".into(), Value::String(prompt.to_string()));
        if let Some(article) = params.get("article") {
            map.insert("article".into(), Value::String(article.to_string()));
        }
        if let Some(commented_code) = params.get("commented_code") {
            map.insert(
                "commented_code".into(),
                Value::String(commented_code.to_string()),
            );
        }
    }

    let deck_card_group = generator
        .create_cards(user.as_ref(), deck.as_thing()?, gpt_card_group)
        .await?;

    manager
        .send_card_group(deck_card_group.card_group.as_ref())
        .await?;
    for card in deck_card_group.card_group.cards.iter() {
        manager.send_card(card.as_ref()).await?;
    }

    handle_show_generic_menu::<CardCommand>(manager).await?;

    Ok(())
}
