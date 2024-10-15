use super::skip_nulls;
use crate::model::card::Card;
use crate::model::tag::Tag;
use crate::model::time::Time;
use crate::model::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CardGroup {
    pub id: Thing,
    pub user: User,

    pub importance: u8,
    pub difficulty: u8,
    pub title: Arc<str>,
    pub data: Option<Arc<Value>>,

    pub time: Time,

    #[serde(deserialize_with = "skip_nulls")]
    pub cards: Vec<Arc<Card>>,

    #[serde(deserialize_with = "skip_nulls")]
    pub tags: Vec<Arc<Tag>>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateCardGroup {
    pub user: Thing,
    pub title: Arc<str>,
    pub importance: u8,
    pub difficulty: u8,
    pub data: Option<Arc<Value>>,
    pub cards: Vec<Thing>,
    pub tags: Vec<Thing>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct UpdateCardGroup {
    pub importance: Option<u8>,
    pub difficulty: Option<u8>,
}
