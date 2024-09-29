use crate::dto::card_group::CardGroupDto;
use crate::dto::deck::DeckDto;
use crate::dto::time::Time;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckCardGroupDto {
    pub id: Thing,

    #[serde(rename = "in")]
    pub deck: Arc<DeckDto>,
    #[serde(rename = "out")]
    pub card_group: Arc<CardGroupDto>,
    
    pub num_answered: Option<usize>,

    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckCardGroupDto {
    pub deck: Thing,
    pub card_group: Thing,
}
