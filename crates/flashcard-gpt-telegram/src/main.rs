#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod command;
pub mod db;
pub mod ext;
pub mod state;

use crate::command::{
    CardCommand, CardGroupCommand, DeckCommand, RootCommand, TagCommand, UserCommand,
};
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingExt;
use crate::ext::bot::BotExt;
use crate::ext::dialogue::DialogueExt;
use crate::state::State;
use flashcard_gpt_core::logging::init_tracing;
use flashcard_gpt_core::reexports::db::engine::remote::ws::{Client, Ws};
use flashcard_gpt_core::reexports::db::opt::auth::Root;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::reexports::trace::{info, span, Level};
use std::str::FromStr;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    info!("Starting dialogue bot...");

    let db: Surreal<Client> = Surreal::init();
    db.connect::<Ws>("127.0.0.1:8477").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("flashcards_gpt").use_db("flashcards").await?;

    let repositories = Repositories::new(db.clone(), span!(Level::INFO, "root"));

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), repositories])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<anyhow::Error> {
    use dptree::case;

    let root_menu_handler = Update::filter_callback_query().endpoint(receive_root_menu_item);

    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(
            case![State::InsideRootMenu]
                .branch(case![RootCommand::Help].endpoint(handle_root_help))
                .branch(case![RootCommand::Start].endpoint(handle_start))
                .branch(case![RootCommand::Deck].endpoint(handle_show_deck_menu))
                .branch(case![RootCommand::User].endpoint(handle_show_user_menu))
                .branch(case![RootCommand::Card].endpoint(handle_show_card_menu))
                .branch(case![RootCommand::Tag].endpoint(handle_show_card_menu))
                .branch(case![RootCommand::CardGroup].endpoint(handle_show_card_group_menu)),
        )
        .branch(case![RootCommand::Cancel].endpoint(cancel));

    let root_message_handler = Update::filter_message()
        .branch(root_command_handler)
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name));

    let deck_command_handler = teloxide::filter_command::<DeckCommand, _>().branch(
        case![State::InsideDeckMenu]
            .branch(case![DeckCommand::Create].endpoint(handle_create_deck)),
    );

    let deck_message_handler = Update::filter_message().branch(deck_command_handler);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(deck_message_handler)
        .branch(root_message_handler)
        .branch(root_menu_handler)
        .branch(Update::filter_message().branch(dptree::endpoint(invalid_state)))
}

async fn handle_create_deck(
    bot: Bot,
    dialogue: FlashGptDialogue,
    repositories: Repositories,
) -> anyhow::Result<()> {
    // repositories.
    bot.send_message(dialogue.chat_id(), "Creating a new deck...")
        .await?;
    Ok(())
}

async fn handle_show_deck_menu(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_menu::<DeckCommand>(dialogue.chat_id()).await?;
    dialogue.set_menu_state::<DeckCommand>().await?;
    Ok(())
}

async fn handle_show_user_menu(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_menu::<UserCommand>(dialogue.chat_id()).await?;
    dialogue.set_menu_state::<UserCommand>().await?;
    Ok(())
}

async fn handle_show_tag_menu(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_menu::<TagCommand>(dialogue.chat_id()).await?;
    dialogue.set_menu_state::<TagCommand>().await?;
    Ok(())
}
async fn handle_show_card_menu(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_menu::<CardCommand>(dialogue.chat_id()).await?;
    dialogue.set_menu_state::<CardCommand>().await?;
    Ok(())
}

async fn handle_show_card_group_menu(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_menu::<CardGroupCommand>(dialogue.chat_id())
        .await?;
    dialogue.set_menu_state::<CardGroupCommand>().await?;
    Ok(())
}

async fn handle_start(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let _binding = repositories
        .bindings
        .get_or_create_telegram_binding(&msg)
        .await?;
    bot.delete_message(msg.chat.id, msg.id).await?;
    bot.send_menu::<RootCommand>(dialogue.chat_id()).await?;
    dialogue.update(State::InsideRootMenu).await?;
    Ok(())
}

async fn handle_root_help(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_help::<RootCommand>(dialogue.chat_id()).await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()> {
    bot.send_message(dialogue.chat_id(), "Cancelling the dialogue.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, dialogue: FlashGptDialogue, msg: Message) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        format!(
            "Unable to handle the message. Type /help to see the usage. Current state: {:?}",
            dialogue.get().await?
        ),
    )
    .await?;
    Ok(())
}

async fn receive_full_name(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    match msg.text().map(ToOwned::to_owned) {
        Some(full_name) => {
            let products = ["Apple", "Banana", "Orange", "Potato"]
                .map(|product| InlineKeyboardButton::callback(product, product));

            bot.send_message(msg.chat.id, "Select a product:")
                .reply_markup(InlineKeyboardMarkup::new([products]))
                .await?;
            dialogue
                .update(State::ReceiveProductChoice { full_name })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your full name.")
                .await?;
        }
    }

    Ok(())
}

async fn receive_root_menu_item(
    bot: Bot,
    dialogue: FlashGptDialogue,
    callback_query: CallbackQuery,
) -> anyhow::Result<()> {
    let state = dialogue.get().await?;
    let Some(menu_item) = &callback_query.data else {
        bot.send_message(
            dialogue.chat_id(),
            "Didn't receive a correct menu item, resetting the dialogue",
        )
        .await?;
        dialogue.update(State::InsideRootMenu).await?;
        return Ok(());
    };

    if let Some(message) = callback_query.message {
        bot.delete_message(message.chat().id, message.id()).await?;
        // bot.edit_message_text(message.chat().id, message.id(), text).await?;
    } else if let Some(id) = callback_query.inline_message_id {
        bot.edit_message_text_inline(id, format!("You chose: {menu_item}"))
            .await?;
        // bot.delete_message(message.chat().id, message.id()).await?;
    }

    info!(?state, menu_item, "Received a menu item");

    match (state, menu_item.as_str()) {
        (None | Some(State::InsideRootMenu), item) if let Ok(cmd) = RootCommand::from_str(item) => {
            match cmd {
                RootCommand::Deck => {
                    handle_show_deck_menu(bot, dialogue).await?;
                }
                RootCommand::User => {
                    handle_show_user_menu(bot, dialogue).await?;
                }
                RootCommand::Card => {
                    handle_show_card_menu(bot, dialogue).await?;
                }
                RootCommand::CardGroup => {
                    handle_show_card_group_menu(bot, dialogue).await?;
                }
                RootCommand::Tag => {
                    handle_show_tag_menu(bot, dialogue).await?;
                }
                RootCommand::Help => {
                    handle_root_help(bot, dialogue).await?;
                }
                RootCommand::Cancel => {
                    cancel(bot, dialogue).await?;
                }
                RootCommand::Start => {
                    // noop
                }
            }
        }
        (Some(State::InsideDeckMenu), item) if let Ok(_cmd) = DeckCommand::from_str(item) => {
            bot.send_message(dialogue.chat_id(), "Deck menu item")
                .await?;
            dialogue.update(State::InsideDeckMenu).await?;
        }
        (_, _) => {}
    }

    // bot.send_message(dialogue.chat_id(), menu_item).await?;
    // dialogue.update(State::ReceiveDeckMenuItem).await?;

    Ok(())
}
