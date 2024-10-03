use crate::dto::deck_card::DeckCardDto;
use crate::dto::deck_card_group::DeckCardGroupDto;
use crate::dto::time::Time;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Duration;
use super::{from_raw_duration_option, to_raw_duration_option};

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct HistoryDto {
    pub id: Thing,

    pub user: Thing,

    pub deck_card: Option<Arc<DeckCardDto>>,
    pub deck_card_group: Option<Arc<DeckCardGroupDto>>,

    #[serde(
        deserialize_with = "from_raw_duration_option",
        serialize_with = "to_raw_duration_option"
    )]
    pub hide_for: Option<Duration>,

    pub difficulty: u8,

    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateHistoryDto {
    pub user: Thing,
    pub deck_card: Option<Thing>,
    pub deck_card_group: Option<Thing>,
    pub difficulty: u8,
    pub time: Option<Time>,

    #[serde(
        deserialize_with = "from_raw_duration_option",
        serialize_with = "to_raw_duration_option"
    )]
    pub hide_for: Option<Duration>,
}
