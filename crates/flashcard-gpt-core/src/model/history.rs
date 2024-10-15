use crate::model::deck_card::DeckCard;
use crate::model::deck_card_group::DeckCardGroup;
use crate::model::time::Time;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Duration;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct HistoryRecord {
    pub id: Thing,

    pub user: Thing,

    pub deck_card: Option<Arc<DeckCard>>,
    pub deck_card_group: Option<Arc<DeckCardGroup>>,

    pub hide_for: Option<Duration>,

    pub difficulty: u8,

    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateHistory {
    pub user: Thing,
    pub deck_card: Option<Thing>,
    pub deck_card_group: Option<Thing>,
    pub difficulty: u8,
    pub time: Option<Time>,
    pub hide_for: Option<Duration>,
}
