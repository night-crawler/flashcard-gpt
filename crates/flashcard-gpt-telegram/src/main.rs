#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod command;
pub mod db;
pub mod ext;

use crate::command::{DeckCommand, RootCommand};
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingExt;
use flashcard_gpt_core::logging::init_tracing;
use flashcard_gpt_core::reexports::db::engine::remote::ws::{Client, Ws};
use flashcard_gpt_core::reexports::db::opt::auth::Root;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::reexports::trace::{info, span, Level};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,

    InsideRootMenu,
    InsideUserMenu,
    InsideDeckMenu,
    InsideCardMenu,
    InsideCardGroupMenu,

    ReceiveFullName,
    ReceiveProductChoice {
        full_name: String,
    },

    ReceiveDeckMenuItem,
}

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

    let command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(
            case![State::Start]
                .branch(case![RootCommand::Help].endpoint(help))
                .branch(case![RootCommand::Start].endpoint(start)),
        )
        .branch(case![RootCommand::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(root_menu_handler)
}

async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let _binding = repositories
        .bindings
        .get_or_create_telegram_binding(&msg)
        .await?;
    let menu_items =
        RootCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()));

    bot.send_message(msg.chat.id, "Root menu:")
        .reply_markup(InlineKeyboardMarkup::new([menu_items]))
        .await?;

    dialogue.update(State::InsideRootMenu).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, RootCommand::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, dialogue: MyDialogue, msg: Message) -> anyhow::Result<()> {
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

async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> anyhow::Result<()> {
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
    dialogue: MyDialogue,
    callback_query: CallbackQuery,
) -> anyhow::Result<()> {
    let state = dialogue.get().await?;
    let Some(menu_item) = &callback_query.data else {
        bot.send_message(
            dialogue.chat_id(),
            "Didn't receive a correct menu item, resetting the dialogue",
        )
        .await?;
        dialogue.update(State::Start).await?;
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

    match (state, menu_item.as_str()) {
        (None | Some(State::Start), item)
            if RootCommand::iter()
                .filter(|cmd| cmd.as_ref() == item)
                .count()
                > 0 => {}
        (_, _) => {}
    }

    bot.send_message(dialogue.chat_id(), menu_item).await?;
    dialogue.update(State::ReceiveDeckMenuItem).await?;

    Ok(())
}
