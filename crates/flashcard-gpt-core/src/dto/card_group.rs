use super::skip_nulls;
use crate::dto::card::CardDto;
use crate::dto::tag::TagDto;
use crate::dto::time::Time;
use crate::dto::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CardGroupDto {
    pub id: Thing,
    pub user: User,

    pub importance: u8,
    pub difficulty: u8,
    pub title: Arc<str>,
    pub data: Option<Arc<Value>>,

    pub time: Time,

    #[serde(deserialize_with = "skip_nulls")]
    pub cards: Vec<Arc<CardDto>>,

    #[serde(deserialize_with = "skip_nulls")]
    pub tags: Vec<Arc<TagDto>>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateCardGroupDto {
    pub user: Thing,
    pub title: Arc<str>,
    pub importance: u8,
    pub difficulty: u8,
    pub data: Option<Arc<Value>>,
    pub cards: Vec<Thing>,
    pub tags: Vec<Thing>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct UpdateCardGroupDto {
    pub importance: Option<u8>,
    pub difficulty: Option<u8>,
}
