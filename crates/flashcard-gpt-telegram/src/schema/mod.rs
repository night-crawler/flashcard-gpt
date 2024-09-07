use crate::invalid_state;
use crate::schema::deck::deck_schema;
use crate::schema::root::{receive_root_menu_item, root_schema};
use crate::state::{FlashGptDialogue, State};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateFilterExt, UpdateHandler};
use teloxide::prelude::{Message, Requester, Update};
use teloxide::{dptree, Bot};

pub mod deck;
pub mod root;

pub fn schema() -> UpdateHandler<anyhow::Error> {
    let root_message_handler = root_schema();
    let deck_message_handler = deck_schema();
    let root_menu_handler = Update::filter_callback_query().endpoint(receive_root_menu_item);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(deck_message_handler)
        .branch(root_message_handler)
        .branch(root_menu_handler)
        .branch(Update::filter_message().branch(dptree::endpoint(invalid_state)))
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
            bot.send_message(msg.chat.id, "Deck description:").await?;
            dialogue
                .update(State::ReceiveDeckDescription { title, tags })
                .await?;
        }
        State::ReceiveDeckDescription { .. } => {}
        State::ReceiveDeckParent {
            title,
            tags,
            description,
        } => {
            bot.send_message(msg.chat.id, "Deck settings / daily limit:").await?;
            dialogue
                .update(State::ReceiveDeckSettings {
                    title,
                    tags,
                    description,
                    parent: None,
                })
                .await?;
        }
        State::ReceiveDeckSettings { .. } => {}
        State::ReceiveDeckConfirm { .. } => {}
    }

    Ok(())
}
