use crate::dto::tag::Tag;
use crate::dto::time::Time;
use crate::dto::user::User;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]

pub struct Settings {
    pub daily_limit: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckDto {
    pub description: Option<Arc<str>>,
    pub parent: Option<Arc<DeckDto>>,
    pub settings: Option<Settings>,
    pub tags: Vec<Tag>,
    pub time: Time,
    pub title: Arc<str>,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDeckDto {
    pub description: Option<Arc<str>>,
    pub parent: Option<Arc<Thing>>,
    pub settings: Option<Settings>,
    pub tags: Vec<Thing>,
    pub title: Arc<str>,
    pub user: Thing,
}
