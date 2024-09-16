use crate::dto::card::CardDto;
use crate::dto::deck::DeckDto;
use crate::dto::time::Time;
use crate::reexports::db::sql::Thing;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use bon::Builder;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckCardDto {
    pub id: Thing,

    #[serde(rename = "in")]
    pub deck: Arc<DeckDto>,
    #[serde(rename = "out")]
    pub card: Arc<CardDto>,

    pub importance: u8,
    pub difficulty: u8,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckCardDto {
    pub deck: Thing,
    pub card: Thing,
    pub importance: u8,
    pub difficulty: u8,
}
