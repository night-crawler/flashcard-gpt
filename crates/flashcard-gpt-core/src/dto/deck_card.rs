use crate::dto::card::CardDto;
use crate::dto::deck::DeckDto;
use crate::dto::time::Time;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckCardDto {
    pub id: Thing,

    #[serde(rename = "in")]
    pub deck: Arc<DeckDto>,
    #[serde(rename = "out")]
    pub card: Arc<CardDto>,

    pub num_answered: Option<usize>,

    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckCardDto {
    pub deck: Thing,
    pub card: Thing,
}
