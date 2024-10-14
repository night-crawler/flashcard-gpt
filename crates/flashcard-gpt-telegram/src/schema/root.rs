use crate::chat_manager::ChatManager;

use crate::command::answer::AnswerCommand;
use crate::command::card::CardCommand;
use crate::command::card_group::CardGroupCommand;
use crate::command::deck::DeckCommand;
use crate::command::ext::CommandExt;
use crate::command::root::RootCommand;
use crate::command::tag::TagCommand;
use crate::command::user::UserCommand;
use crate::schema::answer::{
    handle_cancel_answer, handle_commit_answer, handle_show_article, handle_show_next_card,
    handle_skip_answer,
};
use crate::schema::card::{generate_cards, handle_create_card, handle_generate_cards};
use crate::schema::deck::handle_create_deck;
use crate::schema::receive_next;
use crate::state::bot_state::{BotState, FlashGptDialogue};
use crate::state::state_fields::StateFields;
use anyhow::bail;
use std::str::FromStr;
use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{CallbackQuery, DependencyMap, InlineQuery, Request, Requester, Update};
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::{error, info, warn};

pub fn root_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(
            case![BotState::InsideRootMenu(fields)]
                .branch(case![RootCommand::Help].endpoint(handle_root_help))
                .branch(case![RootCommand::Start].endpoint(handle_start))
                .branch(case![RootCommand::Deck].endpoint(handle_show_generic_menu::<DeckCommand>))
                .branch(case![RootCommand::User].endpoint(handle_show_generic_menu::<UserCommand>))
                .branch(case![RootCommand::Card].endpoint(handle_show_generic_menu::<CardCommand>))
                .branch(case![RootCommand::Tag].endpoint(handle_show_generic_menu::<TagCommand>))
                .branch(
                    case![RootCommand::CardGroup]
                        .endpoint(handle_show_generic_menu::<CardGroupCommand>),
                ),
        )
        .branch(case![RootCommand::Cancel].endpoint(cancel));

    let root_message_handler = Update::filter_message().branch(root_command_handler);
    root_message_handler
}

async fn handle_root_help(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_help::<RootCommand>().await?;
    Ok(())
}

pub async fn cancel(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_message("Cancelling the dialogue.").await?;
    manager.dialogue.exit().await?;
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

async fn handle_start(manager: ChatManager) -> anyhow::Result<()> {
    manager.delete_current_message().await?;
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

#[tracing::instrument(level = "info", skip_all, parent = &manager.span, err, fields(
        chat_id = ?manager.dialogue.chat_id(),
    ))]
pub async fn handle_show_generic_menu<T>(manager: ChatManager) -> anyhow::Result<()>
where
    T: BotCommands + CommandExt,
{
    manager.set_menu_state::<T>().await?;
    manager.send_menu::<T>().await?;
    Ok(())
}

pub(super) async fn receive_root_menu_item(
    manager: ChatManager,
    bot: DefaultParseMode<Bot>,
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
        dialogue
            .update(BotState::InsideRootMenu(StateFields::Empty))
            .await?;
        return Ok(());
    };

    let message = callback_query.regular_message();

    if let Some(message) = message {
        bot.delete_message(message.chat.id, message.id).await?;
    } else if let Some(ref id) = callback_query.inline_message_id {
        bot.edit_message_text_inline(id, format!("You chose: {menu_item}"))
            .await?;
    }

    let user = manager.get_user();

    info!(?state, menu_item, "Received a menu item");

    match (state, menu_item.as_str()) {
        (None | Some(BotState::InsideRootMenu(_)), item)
            if let Ok(cmd) = RootCommand::from_str(item) =>
        {
            match cmd {
                RootCommand::Deck => {
                    handle_show_generic_menu::<DeckCommand>(manager).await?;
                }
                RootCommand::User => {
                    handle_show_generic_menu::<UserCommand>(manager).await?;
                }
                RootCommand::Card => {
                    handle_show_generic_menu::<CardCommand>(manager).await?;
                }
                RootCommand::CardGroup => {
                    handle_show_generic_menu::<CardGroupCommand>(manager).await?;
                }
                RootCommand::Tag => {
                    handle_show_generic_menu::<TagCommand>(manager).await?;
                }
                RootCommand::Help => {
                    handle_root_help(manager).await?;
                }
                RootCommand::Cancel => {
                    cancel(manager).await?;
                }
                RootCommand::Start => {
                    // noop
                }
            }
        }

        (Some(BotState::InsideCardMenu(_)), item) if let Ok(cmd) = CardCommand::from_str(item) => {
            match cmd {
                CardCommand::List => {
                    bot.send_message(dialogue.chat_id(), "Not implemented yet")
                        .await?;
                }
                CardCommand::Create => handle_create_card(manager).await?,
                CardCommand::Generate => handle_generate_cards(manager).await?,
                CardCommand::Next => receive_next(manager).await?,
                CardCommand::Cancel => cancel(manager).await?,
            }
        }

        (Some(BotState::InsideDeckMenu(_)), item) if let Ok(cmd) = DeckCommand::from_str(item) => {
            match cmd {
                DeckCommand::Create => {
                    handle_create_deck(manager).await?;
                }
                DeckCommand::Cancel => {
                    cancel(manager).await?;
                }
                _ => {
                    bot.send_message(dialogue.chat_id(), "Not implemented yet")
                        .await?;
                }
            }
        }
        (Some(BotState::ReceiveDeckTags(mut fields)), tag) => {
            if let Some(tags) = fields.tags_mut() {
                tags.insert(tag.into());
            } else {
                bail!("Invalid state: {:?}", fields);
            }
            manager
                .update_state(BotState::ReceiveDeckTags(fields))
                .await?;
            manager.send_tag_menu().await?;
        }

        (Some(BotState::ReceiveCardTags(mut fields)), tag) => {
            if let Some(tags) = fields.tags_mut() {
                tags.insert(tag.into());
            } else {
                bail!("Invalid state: {:?}", fields);
            }
            manager
                .update_state(BotState::ReceiveCardTags(fields))
                .await?;
            manager.send_tag_menu().await?;
        }

        (Some(BotState::ReceiveDeckParent(mut fields)), next_parent) => {
            if let Some(parent) = fields.parent_mut() {
                parent.replace(next_parent.into());
            } else {
                bail!("Invalid state: {:?}", fields);
            }
            manager
                .update_state(BotState::ReceiveDeckSettingsDailyLimit(fields))
                .await?;
            manager.send_state_and_prompt().await?;
        }

        (Some(BotState::ReceiveCardDeck(mut fields)), next_deck) => {
            if let Some(deck) = fields.deck_mut() {
                deck.replace(next_deck.into());
            } else {
                bail!("Invalid state: {:?}", fields);
            }
            manager
                .update_state(BotState::ReceiveCardConfirm(fields))
                .await?;
            manager.send_state_and_prompt().await?;
        }
        (Some(BotState::ReceiveGenerateCardDeck(mut fields)), next_deck) => {
            if let Some(deck) = fields.deck_mut() {
                deck.replace(next_deck.into());
            } else {
                bail!("Invalid state: {:?}", fields);
            }
            manager
                .update_state(BotState::ReceiveGenerateCardPrompt(fields))
                .await?;
            manager.send_state_and_prompt().await?;
        }
        (Some(BotState::Answering(_)), item) if let Ok(cmd) = AnswerCommand::from_str(item) => {
            match cmd {
                AnswerCommand::Article => handle_show_article(manager).await?,
                AnswerCommand::Next => handle_show_next_card(manager).await?,
                AnswerCommand::Cancel => handle_cancel_answer(manager).await?,
                AnswerCommand::Skip => handle_skip_answer(manager).await?,
                AnswerCommand::Importance(_) => {}
                AnswerCommand::Difficulty(_) => {}
                AnswerCommand::Hide(duration) => {
                    warn!(%duration, "Received an impossible state from menu item");
                }
            }
        }
        (Some(BotState::Answering(_)), item) if let Ok(difficulty) = item.parse::<u8>() => {
            handle_commit_answer(manager, difficulty).await?;
        }
        (Some(BotState::ReceiveGenerateCardConfirm(_)), item)
            if let Ok(CardCommand::Next) = CardCommand::from_str(item) =>
        {
            generate_cards(manager).await?;
        }
        (state, item) => {
            warn!(?state, %item, %user, "No handler for");
        }
    }

    Ok(())
}

pub async fn receive_inline_query(
    bot: DefaultParseMode<Bot>,
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
    .thumbnail_url("https://duckduckgo.com/assets/logo_header.v108.png".parse()?)
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
