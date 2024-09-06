use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::dto::time::Time;
use crate::dto::user::User;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag {
    pub name: Arc<str>,
    pub slug: Arc<str>,
    pub user: User,
    pub time: Time,
}
