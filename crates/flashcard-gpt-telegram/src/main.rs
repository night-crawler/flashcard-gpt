#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(pattern)]

pub mod chat_manager;
pub mod command;
pub mod db;
pub mod ext;
pub mod llm;
pub mod macros;
pub mod message_render;
pub mod schema;
pub mod state;

use crate::db::repositories::Repositories;
use crate::ext::markdown::MarkdownFormatter;
use crate::schema::schema;
use crate::state::{FlashGptDialogue, State};
use flashcard_gpt_core::llm::card_generator::CardGenerator;
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use flashcard_gpt_core::logging::init_tracing;
use flashcard_gpt_core::reexports::db::engine::remote::ws::{Client, Ws};
use flashcard_gpt_core::reexports::db::opt::auth::Root;
use flashcard_gpt_core::reexports::db::Surreal;
use llm_chain::options::{ModelRef, Opt, Options};
use llm_chain::traits::Executor as _;
use llm_chain_openai::chatgpt::Executor;
use markdown::{Constructs, ParseOptions};
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::ParseMode;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use tracing::{info, span, Level};

fn init_card_generator_service(
    repositories: &Repositories,
) -> anyhow::Result<CardGeneratorService> {
    let openai_api_key = std::env::var("OPENAI_API_KEY")?;
    let mut options = Options::builder();
    options.add_option(Opt::ApiKey(openai_api_key));
    options.add_option(Opt::Model(ModelRef::from_model_name("chatgpt-4o-latest")));
    let options = options.build();
    let exec = Executor::new_with_options(options)?;
    let card_generator = CardGenerator::new(exec);

    Ok(CardGeneratorService::new(
        card_generator,
        repositories.cards.clone(),
        repositories.card_groups.clone(),
        repositories.decks.clone(),
        repositories.tags.clone(),
    ))
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
    let card_generation_service = init_card_generator_service(&repositories)?;
    let formatter = MarkdownFormatter::new(ParseOptions {
        constructs: Constructs {
            math_flow: true,
            math_text: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::gfm()
    });

    let span = span!(Level::INFO, "root");

    let bot: DefaultParseMode<Bot> = Bot::from_env().parse_mode(ParseMode::Html);

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            repositories,
            span,
            card_generation_service,
            formatter
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
