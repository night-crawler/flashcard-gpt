use crate::invalid_state;
use crate::schema::deck::deck_schema;
use crate::schema::root::{receive_root_menu_item, root_schema};
use crate::state::State;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::prelude::Update;

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
