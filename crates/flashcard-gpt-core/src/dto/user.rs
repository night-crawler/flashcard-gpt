use crate::dto::time::Time;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use bon::Builder;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct User {
    pub id: Thing,
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserDto {
    pub email: Arc<str>,
    pub name: Arc<str>,
    pub password: Arc<str>,
}


impl From<User> for Thing {
    fn from(value: User) -> Self {
        value.id
    }
}

impl From<&User> for Thing {
    fn from(value: &User) -> Self {
        value.id.clone()
    }
}
