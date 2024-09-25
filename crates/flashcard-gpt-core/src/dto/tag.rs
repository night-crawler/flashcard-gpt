use crate::dto::time::Time;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct TagDto {
    pub id: Thing,
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
    pub time: Time,
}

impl AsRef<TagDto> for TagDto {
    fn as_ref(&self) -> &TagDto {
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct CreateTagDto {
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: Thing,
}

impl From<TagDto> for Thing {
    fn from(value: TagDto) -> Self {
        value.id
    }
}

impl From<&TagDto> for Thing {
    fn from(value: &TagDto) -> Self {
        value.id.clone()
    }
}
