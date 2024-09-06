use crate::dto::time::Time;
use crate::dto::user::User;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use surrealdb::sql::Thing;
use crate::dto::tag::Tag;

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: Thing,
    pub user: User,
    pub title: Arc<String>,
    pub front: Option<Arc<String>>,
    pub back: Option<Arc<String>>,
    pub data: Option<Value>,
    pub hints: Vec<Arc<String>>,
    pub difficulty: i32,
    pub importance: i32,
    pub tags: HashSet<Arc<Tag>>,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardDto {
    pub user: Thing,
    pub title: Arc<String>,
    pub front: Option<Arc<String>>,
    pub back: Option<Arc<String>>,
    pub hints: Vec<Arc<String>>,
    pub difficulty: i32,
    pub importance: i32,
    pub data: Option<Value>,
    pub tags: HashSet<Thing>,
}
