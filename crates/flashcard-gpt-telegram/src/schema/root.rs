use crate::chat_manager::ChatManager;
use crate::command::{CardCommand, CardGroupCommand, CommandExt, DeckCommand, RootCommand, TagCommand, UserCommand};
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingExt;
use crate::ext::bot::BotExt;
use crate::ext::dialogue::DialogueExt;
use crate::schema::deck::handle_create_deck;
use crate::state::{FlashGptDialogue, State};
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::reexports::trace::{error, info};
use std::str::FromStr;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{
    CallbackQuery, DependencyMap, InlineQuery, Message, Request, Requester, Update,
};
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

pub fn root_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(
            case![State::InsideRootMenu]
                .branch(case![RootCommand::Help].endpoint(handle_root_help))
                .branch(case![RootCommand::Start].endpoint(handle_start))
                .branch(case![RootCommand::Deck].endpoint(handle_show_generic_menu::<DeckCommand>))
                .branch(case![RootCommand::User].endpoint(handle_show_generic_menu::<UserCommand>))
                .branch(case![RootCommand::Card].endpoint(handle_show_generic_menu::<CardCommand>))
                .branch(case![RootCommand::Tag].endpoint(handle_show_generic_menu::<TagCommand>))
                .branch(case![RootCommand::CardGroup].endpoint(handle_show_generic_menu::<CardGroupCommand>)),
        )
        .branch(case![RootCommand::Cancel].endpoint(cancel));

    let root_message_handler = Update::filter_message().branch(root_command_handler);
    root_message_handler
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
    bot.set_my_commands(RootCommand::bot_commands()).await?;
    dialogue.set_menu_state::<RootCommand>().await?;
    Ok(())
}


async fn handle_show_generic_menu<T>(bot: Bot, dialogue: FlashGptDialogue) -> anyhow::Result<()>
where
    T: BotCommands + CommandExt,
{
    bot.send_menu::<T>(dialogue.chat_id()).await?;
    bot.set_my_commands(T::bot_commands()).await?;
    dialogue.set_menu_state::<T>().await?;
    Ok(())
}


pub(super) async fn receive_root_menu_item(
    manager: ChatManager,
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

    let message = callback_query.regular_message();

    if let Some(message) = message {
        bot.delete_message(message.chat.id, message.id).await?;
    } else if let Some(ref id) = callback_query.inline_message_id {
        bot.edit_message_text_inline(id, format!("You chose: {menu_item}"))
            .await?;
    }

    info!(?state, menu_item, "Received a menu item");

    match (state, menu_item.as_str()) {
        (None | Some(State::InsideRootMenu), item) if let Ok(cmd) = RootCommand::from_str(item) => {
            match cmd {
                RootCommand::Deck => {
                    handle_show_generic_menu::<DeckCommand>(bot, dialogue).await?;
                }
                RootCommand::User => {
                    handle_show_generic_menu::<UserCommand>(bot, dialogue).await?;
                }
                RootCommand::Card => {
                    handle_show_generic_menu::<CardCommand>(bot, dialogue).await?;
                }
                RootCommand::CardGroup => {
                    handle_show_generic_menu::<CardGroupCommand>(bot, dialogue).await?;
                }
                RootCommand::Tag => {
                    handle_show_generic_menu::<TagCommand>(bot, dialogue).await?;
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
        (Some(State::InsideDeckMenu), item) if let Ok(cmd) = DeckCommand::from_str(item) => {
            match cmd {
                DeckCommand::Create => {
                    handle_create_deck(bot, dialogue).await?;
                }
                _ => {
                    bot.send_message(dialogue.chat_id(), "Not implemented yet")
                        .await?;
                }
            }
        }
        (Some(State::ReceiveDeckTags(mut fields)), tag) => {
            fields.tags.push(tag.into());
            let next_state = State::ReceiveDeckTags(fields);
            manager.update_state(next_state).await?;
            manager.send_tag_menu().await?;
        }
        (
            Some(State::ReceiveDeckParent(mut fields)),
            parent,
        ) => {
            bot.send_message(dialogue.chat_id(), "Deck settings / daily limit:")
                .await?;
            fields.parent = Some(parent.into());
            dialogue
                .update(State::ReceiveDeckSettingsDailyLimit(fields))
                .await?;
        }
        (_, _) => {}
    }

    Ok(())
}

pub async fn receive_inline_query(
    bot: Bot,
    q: InlineQuery,
) -> anyhow::Result<()> {
    info!(?q, "Received an inline query");

    // First, create your actual response
    let google_search = InlineQueryResultArticle::new(
        // Each item needs a unique ID, as well as the response container for the
        // items. These can be whatever, as long as they don't
        // conflict.
        "01".to_string(),
        // What the user will actually see
        "Google Search",
        // What message will be sent when clicked/tapped
        InputMessageContent::Text(InputMessageContentText::new(format!(
            "https://www.google.com/search?q={}",
            q.query,
        ))),
    );
    // While constructing them from the struct itself is possible, it is preferred
    // to use the builder pattern if you wish to add more
    // information to your result. Please refer to the documentation
    // for more detailed information about each field. https://docs.rs/teloxide/latest/teloxide/types/struct.InlineQueryResultArticle.html
    let ddg_search = InlineQueryResultArticle::new(
        "02".to_string(),
        "DuckDuckGo Search".to_string(),
        InputMessageContent::Text(InputMessageContentText::new(format!(
            "https://duckduckgo.com/?q={}",
            q.query
        ))),
    )
        .description("DuckDuckGo Search")
        .thumbnail_url(
            "https://duckduckgo.com/assets/logo_header.v108.png"
                .parse()?,
        )
        .url("https://duckduckgo.com/about".parse()?); // Note: This is the url that will open if they click the thumbnail

    let results = vec![
        InlineQueryResult::Article(google_search),
        InlineQueryResult::Article(ddg_search),
    ];

    // Send it off! One thing to note -- the ID we use here must be of the query
    // we're responding to.
    let response = bot.answer_inline_query(&q.id, results).send().await;
    if let Err(err) = response {
        error!("Error in handler: {:?}", err);
    }

    Ok(())
}
