use crate::model::card_group::CardGroup;
use crate::model::deck::Deck;
use crate::model::time::Time;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckCardGroup {
    pub id: Thing,

    #[serde(rename = "in")]
    pub deck: Arc<Deck>,
    #[serde(rename = "out")]
    pub card_group: Arc<CardGroup>,

    pub num_answered: Option<usize>,

    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckCardGroup {
    pub deck: Thing,
    pub card_group: Thing,
}
