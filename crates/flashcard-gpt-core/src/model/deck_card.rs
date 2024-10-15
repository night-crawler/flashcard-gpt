use crate::model::card::Card;
use crate::model::deck::Deck;
use crate::model::time::Time;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckCard {
    pub id: Thing,

    #[serde(rename = "in")]
    pub deck: Arc<Deck>,
    #[serde(rename = "out")]
    pub card: Arc<Card>,

    pub num_answered: Option<usize>,

    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckCard {
    pub deck: Thing,
    pub card: Thing,
}
