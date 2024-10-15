use super::skip_nulls;
use crate::model::tag::Tag;
use crate::model::time::Time;
use crate::model::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct Card {
    pub id: Thing,
    pub user: Arc<User>,
    pub title: Arc<str>,
    pub front: Option<Arc<str>>,
    pub back: Option<Arc<str>>,
    pub data: Option<Arc<Value>>,
    pub hints: Vec<Arc<str>>,
    pub difficulty: u8,
    pub importance: u8,
    #[serde(deserialize_with = "skip_nulls")]
    pub tags: Vec<Arc<Tag>>,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateCard {
    pub user: Thing,
    pub title: Arc<str>,
    pub front: Option<Arc<str>>,
    pub back: Option<Arc<str>>,
    pub hints: Vec<Arc<str>>,
    pub difficulty: u8,
    pub importance: u8,
    pub data: Option<Arc<Value>>,
    pub tags: Vec<Thing>,
}

impl From<Card> for Thing {
    fn from(value: Card) -> Self {
        value.id
    }
}

impl From<&Card> for Thing {
    fn from(value: &Card) -> Self {
        value.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct UpdateCard {
    pub importance: Option<u8>,
    pub difficulty: Option<u8>,
}
