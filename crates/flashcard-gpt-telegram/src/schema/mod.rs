use crate::schema::deck::deck_schema;
use crate::schema::root::{receive_inline_query, receive_root_menu_item, root_schema};
use crate::state::{FlashGptDialogue, State};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateFilterExt, UpdateHandler};
use teloxide::prelude::{Message, Requester, Update};
use teloxide::{dptree, Bot};
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::reexports::trace::{debug, info};
use crate::db::repositories::Repositories;
use crate::ext::binding::{BindingEntity, BindingExt};
use crate::ext::bot::BotExt;

pub mod deck;
pub mod root;

pub fn schema() -> UpdateHandler<anyhow::Error> {
    let root_message_handler = root_schema();
    let deck_message_handler = deck_schema();
    let root_menu_handler = Update::filter_callback_query().endpoint(receive_root_menu_item);
    let inline_query_handler = Update::filter_inline_query().branch(dptree::endpoint(receive_inline_query));

    let main_branch = dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .map_async(create_binding)
        // .branch(dptree::map_async())
        .branch(deck_message_handler)
        .branch(root_message_handler)
        .branch(root_menu_handler)
        .branch(Update::filter_message().branch(dptree::endpoint(invalid_state)));
    
    dptree::entry()
        .inspect(|update: Update| {
            debug!(?update, "Received update")
        })
        .branch(inline_query_handler)
        .branch(main_branch)
}


async fn create_binding(update: Update, repositories: Repositories) -> anyhow::Result<Option<BindingDto>> {
    let Ok(entity) = BindingEntity::try_from(&update) else {
        return Ok(None);
    };

    let binding = repositories.get_binding(entity).await?;
    info!(?binding, "Binding found");
    Ok(Some(binding))
}

async fn receive_next(
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
            let next_state = State::ReceiveDeckDescription { title, tags };
            let desc = next_state.get_state_description(Some(&msg));
            dialogue.update(next_state).await?;
            bot.send_state_and_prompt(&msg, &desc).await?;
        }
        State::ReceiveDeckDescription { .. } => {}
        State::ReceiveDeckParent {
            title,
            tags,
            description,
        } => {
            let next_state = State::ReceiveDeckSettings {
                title,
                tags,
                description,
                parent: None,
            };
            let desc= next_state.get_state_description(Some(&msg));
            dialogue.update(next_state).await?;
            bot.send_state_and_prompt(&msg, &desc).await?;
        }
        State::ReceiveDeckSettings { .. } => {}
        State::ReceiveDeckConfirm { .. } => {}
    }

    Ok(())
}


pub async fn invalid_state(bot: Bot, dialogue: FlashGptDialogue, msg: Message) -> anyhow::Result<()> {
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