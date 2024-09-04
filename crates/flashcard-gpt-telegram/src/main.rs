#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod db;
pub mod ext;

use crate::db::repositories::Repositories;
use flashcard_gpt_core::dto::binding::{Binding, GetOrCreateBindingDto};
use flashcard_gpt_core::logging::init_tracing;
use flashcard_gpt_core::reexports::db::engine::remote::ws::{Client, Ws};
use flashcard_gpt_core::reexports::db::opt::auth::Root;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::reexports::trace::info;
use flashcard_gpt_core::repo::binding::BindingRepo;
use std::sync::Arc;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;


#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveProductChoice {
        full_name: String,
    },
}


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text.
    Help,
    /// Start the purchase procedure.
    Start,
    /// Cancel the purchase procedure.
    Cancel,
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

    let reposities = Repositories::new(db.clone());

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), reposities])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}


fn schema() -> UpdateHandler<anyhow::Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveProductChoice { full_name }].endpoint(receive_product_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}


async fn get_or_create_user(msg: &Message, repo: &BindingRepo) -> anyhow::Result<Binding> {
    let source_id = Arc::from(if let Some(user) = &msg.from {
        format!("user:{}", user.id)
    } else {
        format!("chat:{}", msg.chat.id)
    });

    let binding = repo.get_binding(Arc::clone(&source_id)).await?;
    if let Some(binding) = binding {
        return Ok(binding);
    }

    let (email, data, name) = if let Some(user) = &msg.from {
        let username = user.username.clone().unwrap_or_else(|| user.id.to_string());
        let serialized = flashcard_gpt_core::reexports::json::to_value(user)?;

        (format!("user-{}@telegram-flash-gpt.example.com", username), serialized, user.full_name())
    } else {
        let serialized = flashcard_gpt_core::reexports::json::to_value(&msg.chat)?;
        let name = msg.chat.title().or_else(|| msg.chat.username()).map(|name| name.to_string()).unwrap_or_else(|| msg.chat.id.to_string());
        (format!("chat-{}telegram-flash-gpt.example.com", msg.chat.id), serialized, name)
    };

    let binding_dto = GetOrCreateBindingDto {
        source_id,
        name: Arc::from(name),
        type_name: Arc::from("telegram"),
        password: Arc::from(uuid::Uuid::new_v4().to_string()),
        data: Some(data),
        email: Arc::from(email),
    };

    Ok(repo.get_or_create_binding(binding_dto).await?)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, repositories: Repositories) -> anyhow::Result<()> {
    let binding = get_or_create_user(&msg, &repositories.bindings).await?;
    info!(?binding, "Started a chat");

    bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Unable to handle the message. Type /help to see the usage.")
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
            dialogue.update(State::ReceiveProductChoice { full_name }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your full name.").await?;
        }
    }

    Ok(())
}

async fn receive_product_selection(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveProductChoice`.
    q: CallbackQuery,
) -> anyhow::Result<()> {
    if let Some(product) = &q.data {
        let text = format!("You chose: {product}");

        bot.answer_callback_query(q.id).text("Processing...").await?;

        if let Some(message) = q.message {
            bot.edit_message_text(message.chat().id, message.id(), text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }


        bot.send_message(
            dialogue.chat_id(),
            format!("{full_name}, product '{product}' has been purchased successfully!"),
        )
            .await?;
        dialogue.exit().await?;
    }

    Ok(())
}