use crate::model::time::Time;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
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
pub struct RegisterUser {
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

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{} {}", self.id, self.name)
    }
}
