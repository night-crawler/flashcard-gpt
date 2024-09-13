use crate::chat_manager::ChatManager;
use crate::command::DeckCommand;
use crate::db::repositories::Repositories;
use crate::schema::receive_next;
use crate::state::{ModifyDeckFields, State};
use crate::FlashGptDialogue;
use anyhow::anyhow;
use flashcard_gpt_core::dto::deck::{CreateDeckDto, Settings};
use flashcard_gpt_core::reexports::db::sql::Thing;
use flashcard_gpt_core::reexports::trace::info;
use std::sync::Arc;
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
        .branch(case![State::ReceiveDeckTitle(fields)].endpoint(receive_deck_title))
        .branch(
            case![State::ReceiveDeckTags(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_tags),
        )
        .branch(case![State::ReceiveDeckDescription(fields)].endpoint(receive_deck_description))
        .branch(
            case![State::ReceiveDeckParent(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_parent),
        )
        .branch(
            case![State::ReceiveDeckSettingsDailyLimit(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_settings),
        )
        .branch(
            case![State::ReceiveDeckConfirm(fields)].branch(
                teloxide::filter_command::<DeckCommand, _>()
                    .branch(case![DeckCommand::Next].endpoint(create_deck)),
            ),
        );

    deck_message_handler
}

pub async fn handle_create_deck(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    let state = dialogue.get_or_default().await?;
    info!(?state, "Handling command in state");

    bot.send_message(
        dialogue.chat_id(),
        "You are creating a new deck.\nUse /cancel to exit and /next to skip the step.\nEnter the title of the deck:",
    )
        .await?;
    dialogue
        .update(State::ReceiveDeckTitle(ModifyDeckFields::default()))
        .await?;
    Ok(())
}

async fn receive_deck_title(manager: ChatManager, msg: Message) -> anyhow::Result<()> {
    let Some(title) = msg.text().map(ToOwned::to_owned) else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let mut fields = manager.get_modify_deck_fields().await?;
    fields.title = Some(Arc::from(title));
    manager.update_state(State::ReceiveDeckTags(fields)).await?;

    manager.send_tag_menu().await?;

    Ok(())
}

async fn receive_deck_tags(manager: ChatManager, msg: Message) -> anyhow::Result<()> {
    let Some(new_tags) = msg.text().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .map(Arc::from)
            .collect::<Vec<_>>()
    }) else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let mut fields = manager.get_modify_deck_fields().await?;
    fields.tags.extend(new_tags.clone());
    manager.update_state(State::ReceiveDeckTags(fields)).await?;
    manager.send_tag_menu().await?;
    Ok(())
}

async fn receive_deck_description(manager: ChatManager, msg: Message) -> anyhow::Result<()> {
    let Some(description) = msg.text().map(ToOwned::to_owned) else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let mut fields = manager.get_modify_deck_fields().await?;
    fields.description = Some(Arc::from(description));
    manager
        .update_state(State::ReceiveDeckParent(fields))
        .await?;

    manager.send_deck_menu().await?;

    Ok(())
}

async fn receive_deck_parent(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_invalid_input().await?;
    Ok(())
}

async fn receive_deck_settings(manager: ChatManager, msg: Message) -> anyhow::Result<()> {
    let Some(Ok(daily_limit)) = msg
        .text()
        .map(ToOwned::to_owned)
        .map(|s| s.parse::<usize>())
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let mut fields = manager.get_modify_deck_fields().await?;
    fields.daily_limit = Some(daily_limit);

    manager.send_state_and_prompt().await?;

    Ok(())
}

async fn create_deck(
    manager: ChatManager,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let fields = manager.get_modify_deck_fields().await?;

    let parent = if let Some(parent) = fields.parent {
        let parent =
            Thing::try_from(parent.as_ref()).map_err(|_| anyhow!("Failed to get parent by id"))?;
        repositories.decks.get_by_id(parent).await?.id.into()
    } else {
        None
    };

    let user_id = manager.binding.user.id.clone();

    let tags = repositories
        .get_or_create_tags(user_id.clone(), fields.tags)
        .await?
        .into_iter()
        .map(|tag| tag.id)
        .collect();

    let title = fields
        .title
        .ok_or_else(|| anyhow!("Title was not provided"))?;

    let deck = repositories
        .decks
        .create(CreateDeckDto {
            title,
            description: fields.description,
            parent,
            user: user_id,
            tags,
            settings: fields
                .daily_limit
                .map(|limit| Settings { daily_limit: limit }),
        })
        .await?;

    manager
        .bot
        .send_message(msg.chat.id, format!("Created a new deck: {deck:?}"))
        .await?;

    dialogue.exit().await?;
    Ok(())
}
