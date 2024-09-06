#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod command;
pub mod db;
pub mod ext;
pub mod schema;
pub mod state;

use crate::db::repositories::Repositories;
use crate::schema::schema;
use crate::state::{FlashGptDialogue, State};
use flashcard_gpt_core::logging::init_tracing;
use flashcard_gpt_core::reexports::db::engine::remote::ws::{Client, Ws};
use flashcard_gpt_core::reexports::db::opt::auth::Root;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::reexports::trace::{info, span, Level};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

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

async fn receive_deck_continue(
    bot: Bot,
    dialogue: FlashGptDialogue,
    msg: Message,
) -> anyhow::Result<()> {
    let Some(state) = dialogue.get().await? else {
        bot.send_message(
            msg.chat.id,
            "No state found, resetting the dialogue to the root menu.",
        )
        .await?;
        dialogue.update(State::InsideRootMenu).await?;
        return Ok(());
    };

    match state {
        State::InsideRootMenu => {}
        State::InsideUserMenu => {}
        State::InsideDeckMenu => {}
        State::InsideCardMenu => {}
        State::InsideCardGroupMenu => {}
        State::InsideTagMenu => {}
        State::ReceiveDeckTitle => {}
        State::ReceiveDeckTags { tags, title } => {
            bot.send_message(msg.chat.id, "Deck description:").await?;
            dialogue
                .update(State::ReceiveDeckDescription { title, tags })
                .await?;
        }
        State::ReceiveDeckDescription { .. } => {}
    }

    Ok(())
}
