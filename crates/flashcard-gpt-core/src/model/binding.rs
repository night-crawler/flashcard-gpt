use crate::model::time::Time;
use crate::model::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct Binding {
    pub id: Thing,
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub data: Option<Arc<serde_json::Value>>,
    pub user: Arc<User>,
    pub time: Time,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct GetOrCreateBinding {
    pub source_id: Arc<str>,
    pub type_name: Arc<str>,
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
    pub data: Option<serde_json::Value>,
}

impl From<Binding> for Thing {
    fn from(value: Binding) -> Self {
        value.id
    }
}

impl From<&Binding> for Thing {
    fn from(value: &Binding) -> Self {
        value.id.clone()
    }
}
