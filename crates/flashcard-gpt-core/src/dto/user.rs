use crate::dto::time::Time;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Thing>,
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
    pub time: Option<Time>,
}
