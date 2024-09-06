use crate::dto::time::Time;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TagDto {
    pub id: Thing,
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CreateTagDto {
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
}
