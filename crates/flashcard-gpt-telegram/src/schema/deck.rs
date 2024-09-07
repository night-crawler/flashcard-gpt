use crate::command::DeckCommand;
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingExt;
use crate::schema::receive_next;
use crate::state::State;
use crate::FlashGptDialogue;
use flashcard_gpt_core::dto::deck::{CreateDeckDto, Settings};
use flashcard_gpt_core::reexports::db::RecordId;
use std::str::FromStr;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{DependencyMap, Message, Requester, Update};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;
use crate::ext::bot::BotExt;

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

async fn handle_create_deck(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_message(dialogue.chat_id(), "Deck name:").await?;
    dialogue.update(State::ReceiveDeckTitle).await?;
    Ok(())
}

async fn receive_deck_tags(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    (title, mut tags): (String, Vec<String>),
) -> anyhow::Result<()> {
    let Some(new_tags) = msg.text().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_owned())
            .collect::<Vec<_>>()
    }) else {
        bot.send_message(msg.chat.id, "Please, send the deck tags.")
            .await?;
        return Ok(());
    };

    tags.extend(new_tags);
    bot.send_message(msg.chat.id, format!("Received tags: {tags:?}"))
        .await?;
    dialogue
        .update(State::ReceiveDeckTags { title, tags })
        .await?;

    Ok(())
}

async fn receive_deck_title(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    let Some(deck_title) = msg.text().map(ToOwned::to_owned) else {
        bot.send_message(msg.chat.id, "Please, send the deck title.")
            .await?;
        return Ok(());
    };

    let items = ["a", "b", "c"]
        .into_iter()
        .map(|cmd| InlineKeyboardButton::callback(cmd, cmd));
    bot.send_message(msg.chat.id, "Deck tags:")
        .reply_markup(InlineKeyboardMarkup::new([items]))
        .await?;

    dialogue
        .update(State::ReceiveDeckTags {
            title: deck_title,
            tags: vec![],
        })
        .await?;

    Ok(())
}

async fn receive_deck_description(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    (title, tags): (String, Vec<String>),
    repositories: Repositories,
) -> anyhow::Result<()> {
    let Some(description) = msg.text().map(ToOwned::to_owned) else {
        bot.send_message(msg.chat.id, "Please, send deck description.")
            .await?;
        return Ok(());
    };
    
    let binding = repositories
        .bindings
        .get_or_create_telegram_binding(&msg)
        .await?;

    let decks = repositories
        .decks
        .list_by_user_id(binding.user.id.clone())
        .await?;
    
    bot.send_decks_menu(msg.chat.id, decks).await?;

    dialogue
        .update(State::ReceiveDeckParent {
            title,
            tags,
            description,
        })
        .await?;

    Ok(())
}

async fn receive_deck_parent(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    (title, tags, description): (String, Vec<String>, String),
) -> anyhow::Result<()> {
    match msg.text().map(ToOwned::to_owned) {
        Some(_parent) => {
            bot.send_message(msg.chat.id, "Deck settings / daily limit:")
                .await?;
            dialogue
                .update(State::ReceiveDeckSettings {
                    title,
                    tags,
                    description,
                    parent: None,
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send deck description.")
                .await?;
        }
    }

    Ok(())
}

async fn receive_deck_settings(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    (title, tags, description, parent): (String, Vec<String>, String, Option<String>),
) -> anyhow::Result<()> {
    let Some(Ok(daily_limit)) = msg
        .text()
        .map(ToOwned::to_owned)
        .map(|s| s.parse::<usize>())
    else {
        bot.send_message(
            msg.chat.id,
            "Please, send the daily limit. It must be a number.",
        )
        .await?;
        return Ok(());
    };
    
    bot.send_message(msg.chat.id, "Confirm the deck creation")
        .await?;
    
    dialogue
        .update(State::ReceiveDeckConfirm {
            title,
            tags,
            description,
            parent,
            daily_limit: Some(daily_limit),
        })
        .await?;

    Ok(())
}

async fn create_deck(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    (title, tags, description, parent, daily_limit): (
        String,
        Vec<String>,
        String,
        Option<String>,
        Option<usize>,
    ),
    repositories: Repositories,
) -> anyhow::Result<()> {
    let binding = repositories
        .bindings
        .get_or_create_telegram_binding(&msg)
        .await?;

    let parent = if let Some(parent) = parent {
        Some(
            repositories
                .decks
                .get_by_id(RecordId::from_str(&parent)?)
                .await?
                .id,
        )
    } else {
        None
    };

    let deck = repositories
        .decks
        .create(CreateDeckDto {
            title: title.into(),
            description: Some(description.into()),
            parent,
            user: binding.user.id.clone(),
            tags: vec![],
            settings: daily_limit.map(|limit| Settings { daily_limit: limit }),
        })
        .await?;

    bot.send_message(msg.chat.id, format!("Created a new deck: {deck:?}"))
        .await?;
    dialogue.exit().await?;
    Ok(())
}
