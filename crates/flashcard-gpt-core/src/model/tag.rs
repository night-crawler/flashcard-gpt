use crate::model::time::Time;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct Tag {
    pub id: Thing,
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
    pub time: Time,
}

impl AsRef<Tag> for Tag {
    fn as_ref(&self) -> &Tag {
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct CreateTag {
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
}

impl From<Tag> for Thing {
    fn from(value: Tag) -> Self {
        value.id
    }
}

impl From<&Tag> for Thing {
    fn from(value: &Tag) -> Self {
        value.id.clone()
    }
}
