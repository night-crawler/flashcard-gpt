use crate::dto::tag::TagDto;
use crate::dto::time::Time;
use crate::dto::user::User;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub daily_limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckDto {
    pub id: Thing,
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<Settings>,
    pub tags: Vec<TagDto>,
    pub time: Time,
    pub title: Arc<str>,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDeckDto {
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<Settings>,
    pub tags: Vec<Thing>,
    pub title: Arc<str>,
    pub user: Thing,
}
