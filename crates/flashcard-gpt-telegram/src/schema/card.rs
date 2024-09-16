use crate::chat_manager::ChatManager;
use crate::command::CardCommand;
use crate::db::repositories::Repositories;
use crate::schema::receive_next;
use crate::state::{State, StateFields};
use crate::{patch_state, FlashGptDialogue};
use anyhow::anyhow;

use flashcard_gpt_core::dto::card::CreateCardDto;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Update};

pub fn card_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let card_command_handler = teloxide::filter_command::<CardCommand, _>().branch(
        case![State::InsideCardMenu(fields)]
            .branch(case![CardCommand::Create].endpoint(handle_create_card)),
    );

    let card_message_handler = Update::filter_message()
        .branch(card_command_handler)
        .branch(case![State::ReceiveCardTitle(fields)].endpoint(receive_card_title))
        .branch(case![State::ReceiveCardFront(fields)].endpoint(receive_card_front))
        .branch(case![State::ReceiveCardBack(fields)].endpoint(receive_card_back))
        .branch(case![State::ReceiveCardHints(fields)].endpoint(receive_card_hints))
        .branch(case![State::ReceiveCardDifficulty(fields)].endpoint(receive_card_difficulty))
        .branch(case![State::ReceiveCardImportance(fields)].endpoint(receive_card_importance))
        .branch(
            case![State::ReceiveCardTags(fields)]
                .branch(
                    teloxide::filter_command::<CardCommand, _>()
                        .branch(case![CardCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_card_tags),
        )
        .branch(
            case![State::ReceiveCardConfirm(fields)].branch(
                teloxide::filter_command::<CardCommand, _>()
                    .branch(case![CardCommand::Next].endpoint(create_card)),
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
        .update_state(State::ReceiveCardTitle(StateFields::default_card()))
        .await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_card_title(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_title) = manager.parse_text() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };
    let fields = patch_state!(
        manager,
        StateFields::Card { title },
        |title: &mut Option<Arc<str>>| { title.replace(next_title) }
    );

    manager
        .update_state(State::ReceiveCardFront(fields))
        .await?;
    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn receive_card_front(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_front) = manager.parse_text() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Card { front },
        |front: &mut Option<Arc<str>>| { front.replace(next_front) }
    );
    manager.update_state(State::ReceiveCardBack(fields)).await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_card_back(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_back) = manager.parse_text() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(manager, StateFields::Card { back }, |back: &mut Option<
        Arc<str>,
    >| {
        back.replace(next_back)
    });

    manager
        .update_state(State::ReceiveCardHints(fields))
        .await?;
    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn receive_card_hints(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_hints) = manager.parse_comma_separated_values() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(manager, StateFields::Card { hints }, |hints: &mut Vec<
        Arc<str>,
    >| {
        hints.extend(next_hints)
    });
    manager
        .update_state(State::ReceiveCardDifficulty(fields))
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
        .update_state(State::ReceiveCardImportance(fields))
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

    manager.update_state(State::ReceiveCardTags(fields)).await?;
    manager.send_tag_menu().await?;
    Ok(())
}

async fn receive_card_tags(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_tags) = manager.parse_comma_separated_values() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(manager, StateFields::Card { tags }, |tags: &mut Vec<
        Arc<str>,
    >| {
        tags.extend(next_tags)
    });
    manager.update_state(State::ReceiveCardTags(fields)).await?;
    manager.send_state_and_prompt().await?;
    manager.send_tag_menu().await?;

    Ok(())
}

async fn create_card(
    manager: ChatManager,
    dialogue: FlashGptDialogue,
    repositories: Repositories,
) -> anyhow::Result<()> {
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
    } = manager.get_state().await?.take_fields()
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let user_id = manager.binding.user.id.clone();

    let tags = repositories
        .get_or_create_tags(user_id.clone(), tags)
        .await?
        .into_iter()
        .map(|tag| tag.id)
        .collect();

    let title = title.ok_or_else(|| anyhow!("Title was not provided"))?;

    let card = repositories
        .cards
        .create(CreateCardDto {
            user: user_id,
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

    dialogue.exit().await?;
    Ok(())
}
