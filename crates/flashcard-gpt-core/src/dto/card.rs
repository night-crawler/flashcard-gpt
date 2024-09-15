use crate::dto::tag::TagDto;
use crate::dto::time::Time;
use crate::dto::user::User;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: Thing,
    pub user: Arc<User>,
    pub title: Arc<String>,
    pub front: Option<Arc<str>>,
    pub back: Option<Arc<str>>,
    pub data: Option<Arc<Value>>,
    pub hints: Vec<Arc<str>>,
    pub difficulty: u8,
    pub importance: u8,
    pub tags: HashSet<Arc<TagDto>>,
    pub time: Option<Time>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardDto {
    pub user: Thing,
    pub title: Arc<str>,
    pub front: Option<Arc<str>>,
    pub back: Option<Arc<str>>,
    pub hints: Vec<Arc<str>>,
    pub difficulty: u8,
    pub importance: u8,
    pub data: Option<Arc<Value>>,
    pub tags: HashSet<Thing>,
}
