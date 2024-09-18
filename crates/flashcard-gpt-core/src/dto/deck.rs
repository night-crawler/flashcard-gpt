use super::skip_nulls;
use crate::dto::tag::TagDto;
use crate::dto::time::Time;
use crate::dto::user::User;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckSettings {
    pub daily_limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct DeckDto {
    pub id: Thing,
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<DeckSettings>,
    #[serde(deserialize_with = "skip_nulls")]
    pub tags: Vec<Arc<TagDto>>,
    pub time: Time,
    pub title: Arc<str>,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateDeckDto {
    pub description: Option<Arc<str>>,
    pub parent: Option<Thing>,
    pub settings: Option<DeckSettings>,
    pub tags: Vec<Thing>,
    pub title: Arc<str>,
    pub user: Thing,
}

impl From<DeckDto> for Thing {
    fn from(value: DeckDto) -> Self {
        value.id
    }
}

impl From<&DeckDto> for Thing {
    fn from(value: &DeckDto) -> Self {
        value.id.clone()
    }
}
