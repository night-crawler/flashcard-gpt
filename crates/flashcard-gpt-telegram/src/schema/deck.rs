use crate::command::DeckCommand;
use crate::db::repositories::Repositories;
use crate::ext::bot::BotExt;
use crate::ext::dialogue::DialogueExt;
use crate::schema::receive_next;
use crate::state::State;
use crate::{patch_state, propagate, FlashGptDialogue};
use anyhow::anyhow;
use flashcard_gpt_core::dto::deck::{CreateDeckDto, Settings};
use flashcard_gpt_core::reexports::db::sql::Thing;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Message, Requester, Update};
use teloxide::Bot;

pub fn deck_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let deck_command_handler = teloxide::filter_command::<DeckCommand, _>().branch(
        case![State::InsideDeckMenu]
            .branch(case![DeckCommand::Create].endpoint(handle_create_deck)),
    );

    let deck_message_handler = Update::filter_message()
        .branch(deck_command_handler)
        .branch(case![State::ReceiveDeckTitle].endpoint(receive_deck_title))
        .branch(
            case![State::ReceiveDeckTags { title, tags }]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_tags),
        )
        .branch(
            case![State::ReceiveDeckDescription { title, tags }].endpoint(receive_deck_description),
        )
        .branch(
            case![State::ReceiveDeckParent {
                title,
                tags,
                description
            }]
            .branch(
                teloxide::filter_command::<DeckCommand, _>()
                    .branch(case![DeckCommand::Next].endpoint(receive_next)),
            )
            .endpoint(receive_deck_parent),
        )
        .branch(
            case![State::ReceiveDeckSettings {
                title,
                tags,
                description,
                parent
            }]
            .branch(
                teloxide::filter_command::<DeckCommand, _>()
                    .branch(case![DeckCommand::Next].endpoint(receive_next)),
            )
            .endpoint(receive_deck_settings),
        )
        .branch(
            case![State::ReceiveDeckConfirm {
                title,
                tags,
                description,
                parent,
                daily_limit
            }]
            .branch(
                teloxide::filter_command::<DeckCommand, _>()
                    .branch(case![DeckCommand::Next].endpoint(create_deck)),
            ),
        );

    deck_message_handler
}

pub async fn handle_create_deck(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_message(
        dialogue.chat_id(),
        "You are creating a new deck.\nUse /cancel to exit and /next to skip the step.\nEnter the title of the deck:",
    )
    .await?;
    dialogue.update(State::ReceiveDeckTitle).await?;
    Ok(())
}

async fn receive_deck_title(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;
    let Some(title) = msg.text().map(ToOwned::to_owned) else {
        bot.send_invalid_input(&msg, &desc).await?;
        return Ok(());
    };

    let next_state = State::ReceiveDeckTags {
        title,
        tags: Vec::new(),
    };

    let desc = next_state.get_state_description(Some(&msg));
    let binding = repositories.get_binding(&msg).await?;

    let tag_menu = repositories.build_tag_menu(binding.user.id.clone()).await?;
    bot.send_state_and_prompt_with_keyboard(&msg, &desc, tag_menu)
        .await?;

    dialogue.update(next_state).await?;

    Ok(())
}

async fn receive_deck_tags(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;

    let Some(new_tags) = msg.text().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    }) else {
        bot.send_message(msg.chat.id, desc.invalid_input.as_ref())
            .await?;
        return Ok(());
    };
    let next_state = patch_state!(
        dialogue,
        State::ReceiveDeckTags { tags },
        |tags: &mut Vec<String>| {
            tags.extend(new_tags);
        }
    );
    let desc = next_state.get_state_description(Some(&msg));
    let binding = repositories.get_binding(&msg).await?;

    let tag_menu = repositories.build_tag_menu(binding.user.id.clone()).await?;
    bot.send_state_and_prompt_with_keyboard(&msg, &desc, tag_menu)
        .await?;

    Ok(())
}

async fn receive_deck_description(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;
    let Some(description) = msg.text().map(ToOwned::to_owned) else {
        bot.send_invalid_input(&msg, &desc).await?;
        return Ok(());
    };

    let next = propagate!(
        dialogue,
        State::ReceiveDeckDescription { title, tags },
        State::ReceiveDeckParent {
            description: description
        }
    );

    let desc = next.get_state_description(Some(&msg));

    let binding = repositories.get_binding(&msg).await?;
    let parent_menu = repositories
        .build_deck_menu(binding.user.id.clone())
        .await?;
    bot.send_state_and_prompt_with_keyboard(&msg, &desc, parent_menu)
        .await?;

    Ok(())
}

async fn receive_deck_parent(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;
    bot.send_invalid_input(&msg, &desc).await?;
    Ok(())
}

async fn receive_deck_settings(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;
    let Some(Ok(daily_limit)) = msg
        .text()
        .map(ToOwned::to_owned)
        .map(|s| s.parse::<usize>())
    else {
        bot.send_invalid_input(&msg, &desc).await?;
        return Ok(());
    };

    let next_state = propagate!(
        dialogue,
        State::ReceiveDeckSettings {
            title,
            tags,
            description,
            parent
        },
        State::ReceiveDeckConfirm {
            daily_limit: Some(daily_limit)
        }
    );
    let desc = &next_state.get_state_description(Some(&msg));
    bot.send_state_and_prompt(&msg, desc).await?;

    Ok(())
}

async fn create_deck(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let desc = dialogue.get_current_state_description(Some(&msg)).await?;
    let State::ReceiveDeckConfirm {
        title,
        tags,
        description,
        parent,
        daily_limit,
    } = dialogue.get_or_default().await?
    else {
        bot.send_invalid_input(&msg, &desc).await?;
        return Ok(());
    };

    let parent = if let Some(parent) = parent {
        let parent = Thing::try_from(parent).map_err(|_| anyhow!("Failed to get parent by id"))?;
        repositories.decks.get_by_id(parent).await?.id.into()
    } else {
        None
    };

    let binding = repositories.get_binding(&msg).await?;

    let tags = repositories
        .get_or_create_tags(binding.user.id.clone(), tags)
        .await?
        .into_iter()
        .map(|tag| tag.id)
        .collect();

    let deck = repositories
        .decks
        .create(CreateDeckDto {
            title: title.into(),
            description: Some(description.into()),
            parent,
            user: binding.user.id.clone(),
            tags,
            settings: daily_limit.map(|limit| Settings { daily_limit: limit }),
        })
        .await?;

    bot.send_message(msg.chat.id, format!("Created a new deck: {deck:?}"))
        .await?;
    dialogue.exit().await?;
    Ok(())
}
