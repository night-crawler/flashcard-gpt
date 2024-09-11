use crate::command::{
    CardCommand, CardGroupCommand, DeckCommand, RootCommand, TagCommand, UserCommand,
};
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingExt;
use crate::ext::bot::BotExt;
use crate::ext::dialogue::DialogueExt;
use crate::state::{FlashGptDialogue, State};
use flashcard_gpt_core::reexports::trace::info;
use std::str::FromStr;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{CallbackQuery, DependencyMap, Message, Requester, Update};
use teloxide::Bot;
use crate::patch_state;
use crate::schema::deck::handle_create_deck;

pub fn root_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
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
        .get_or_create_telegram_binding((&msg).into())
        .await?;
    bot.delete_message(msg.chat.id, msg.id).await?;
    bot.send_menu::<RootCommand>(dialogue.chat_id()).await?;
    dialogue.update(State::InsideRootMenu).await?;
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

pub(super) async fn receive_root_menu_item(
    bot: Bot,
    dialogue: FlashGptDialogue,
    callback_query: CallbackQuery,
    repositories: Repositories,
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
    } else if let Some(id) = callback_query.inline_message_id {
        bot.edit_message_text_inline(id, format!("You chose: {menu_item}"))
            .await?;
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
        (Some(State::InsideDeckMenu), item) if let Ok(cmd) = DeckCommand::from_str(item) => {
            match cmd {
                DeckCommand::Create => {
                    handle_create_deck(bot, dialogue).await?;
                }
                _ => {}
            }
        }
        (Some(State::ReceiveDeckTags { title, tags }), tag) => {
            info!(?tags, tag, "Received a tag");
            let next_state = patch_state!(
                dialogue,
                State::ReceiveDeckTags { tags },
                |existing: &mut Vec<String>| {
                    existing.push(tag.to_string());
                }
            );
            let desc = next_state.get_state_description(None);
            bot.send_message(dialogue.chat_id(), format!("Received tags: {tags:?}"))
                .await?;
        }
        (Some(State::ReceiveDeckParent { title, tags, description }), parent) => {
            bot.send_message(dialogue.chat_id(), "Deck settings / daily limit:")
                .await?;
            dialogue
                .update(State::ReceiveDeckSettings {
                    title,
                    tags,
                    description,
                    parent: Some(parent.to_owned()),
                })
                .await?;
        }
        (_, _) => {}
    }

    // bot.send_message(dialogue.chat_id(), menu_item).await?;
    // dialogue.update(State::ReceiveDeckMenuItem).await?;

    Ok(())
}
