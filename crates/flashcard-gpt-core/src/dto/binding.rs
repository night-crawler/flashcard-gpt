use crate::dto::time::Time;
use crate::dto::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct BindingDto {
    pub id: Thing,
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub data: Option<Arc<serde_json::Value>>,
    pub user: Arc<User>,
    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct GetOrCreateBindingDto {
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
    pub data: Option<serde_json::Value>,
}

impl From<BindingDto> for Thing {
    fn from(value: BindingDto) -> Self {
        value.id
    }
}

impl From<&BindingDto> for Thing {
    fn from(value: &BindingDto) -> Self {
        value.id.clone()
    }
}
