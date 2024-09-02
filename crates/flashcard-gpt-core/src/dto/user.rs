use crate::dto::time::Time;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Option<RecordId>,
    pub email: Arc<String>,
    pub name: Arc<String>,
    pub password: Arc<String>,
    pub time: Option<Time>,
}
