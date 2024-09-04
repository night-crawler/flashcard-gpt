use std::sync::Arc;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use crate::dto::time::Time;
use crate::dto::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Binding {
    pub id: Thing,
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub data: Option<serde_json::Value>,
    pub user: Arc<User>,
    pub time: Time,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrCreateBindingDto {
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
    pub data: Option<serde_json::Value>,
}
