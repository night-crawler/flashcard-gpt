use super::skip_nulls;
use crate::model::tag::Tag;
use crate::model::time::Time;
use crate::model::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckSettings {
    pub daily_limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct Deck {
    pub id: Thing,
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<DeckSettings>,
    #[serde(deserialize_with = "skip_nulls")]
    pub tags: Vec<Arc<Tag>>,
    pub time: Time,
    pub title: Arc<str>,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeck {
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<DeckSettings>,
    pub tags: Vec<Thing>,
    pub title: Arc<str>,
    pub user: Thing,
}

impl From<Deck> for Thing {
    fn from(value: Deck) -> Self {
        value.id
    }
}

impl From<&Deck> for Thing {
    fn from(value: &Deck) -> Self {
        value.id.clone()
    }
}
