use teloxide::Bot;
use crate::command::DeckCommand;
use crate::state::State;
use crate::{ receive_deck_continue,   FlashGptDialogue};
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{DependencyMap, Message, Requester, Update};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
                        .branch(case![DeckCommand::Continue].endpoint(receive_deck_continue)),
                )
                .endpoint(receive_deck_tags),
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
    match msg.text().map(|s| {
        s.split(',')
            .map(|s| s.trim().to_owned())
            .collect::<Vec<_>>()
    }) {
        Some(new_tags) => {
            tags.extend(new_tags);
            bot.send_message(msg.chat.id, format!("Received tags: {tags:?}"))
                .await?;
            dialogue
                .update(State::ReceiveDeckTags { title, tags })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send the deck tags.")
                .await?;
        }
    }

    Ok(())
}

async fn receive_deck_title(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    match msg.text().map(ToOwned::to_owned) {
        Some(deck_title) => {
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
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send the deck name.")
                .await?;
        }
    }

    Ok(())
}
