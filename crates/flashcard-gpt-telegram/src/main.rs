#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(anonymous_lifetime_in_impl_trait)]
pub mod command;
pub mod db;
pub mod ext;
pub mod macros;
pub mod schema;
pub mod state;
pub mod chat_manager;

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

    let span = span!(Level::INFO, "root");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), repositories, span])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
