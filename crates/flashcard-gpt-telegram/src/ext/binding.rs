use flashcard_gpt_core::dto::binding::{BindingDto, GetOrCreateBindingDto};
use flashcard_gpt_core::error::CoreError;
use flashcard_gpt_core::repo::binding::BindingRepo;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{Chat, User};

pub trait BindingExt {
    fn get_or_create_telegram_binding(
        &self,
        entity: impl Into<BindingEntity<'_>>,
    ) -> impl Future<Output = Result<BindingDto, CoreError>>;
}

impl BindingExt for BindingRepo {
    async fn get_or_create_telegram_binding(
        &self,
        entity: impl Into<BindingEntity<'_>>,
    ) -> Result<BindingDto, CoreError> {
        let entity = entity.into();
        let source_id = entity.id();

        let binding = self.get_by_source_id(Arc::clone(&source_id)).await?;
        if let Some(binding) = binding {
            return Ok(binding);
        }

        let binding_dto = GetOrCreateBindingDto {
            source_id,
            name: entity.name(),
            type_name: Arc::from("telegram"),
            password: Arc::from(uuid::Uuid::new_v4().to_string()),
            data: Some(entity.data()),
            email: entity.email(),
        };

        self.get_or_create_binding(binding_dto).await
    }
}

#[derive(Debug, Clone)]
pub enum BindingEntity<'a> {
    User(&'a User),
    Chat(&'a Chat),
}

impl<'a> BindingEntity<'a> {
    pub fn id(&self) -> Arc<str> {
        BindingIdentity::from(self).to_string().into()
    }

    pub fn name(&self) -> Arc<str> {
        match self {
            BindingEntity::User(user) => user.full_name().into(),
            BindingEntity::Chat(chat) => chat
                .title()
                .or_else(|| chat.username())
                .map(|name| name.to_string())
                .unwrap_or_else(|| chat.id.to_string())
                .into(),
        }
    }

    pub fn data(&self) -> flashcard_gpt_core::reexports::json::Value {
        match self {
            BindingEntity::User(user) => {
                flashcard_gpt_core::reexports::json::to_value(user).unwrap()
            }
            BindingEntity::Chat(chat) => {
                flashcard_gpt_core::reexports::json::to_value(chat).unwrap()
            }
        }
    }

    pub fn email(&self) -> Arc<str> {
        match self {
            BindingEntity::User(user) => {
                let username = user.username.clone().unwrap_or_else(|| user.id.to_string());
                format!("user-{username}@telegram-flash-gpt.example.com")
            }
            BindingEntity::Chat(chat) => {
                format!("chat-{}telegram-flash-gpt.example.com", chat.id)
            }
        }
        .into()
    }
}

impl<'a, 'b> From<&'b Message> for BindingEntity<'a>
where
    'b: 'a,
{
    fn from(value: &'b Message) -> Self {
        if let Some(user) = &value.from {
            BindingEntity::User(user)
        } else {
            BindingEntity::Chat(&value.chat)
        }
    }
}

impl<'a, 'b> From<&'b User> for BindingEntity<'a>
where
    'b: 'a,
{
    fn from(value: &'b User) -> Self {
        BindingEntity::User(value)
    }
}

impl<'a, 'b> From<&'b Chat> for BindingEntity<'a>
where
    'b: 'a,
{
    fn from(value: &'b Chat) -> Self {
        BindingEntity::Chat(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BindingIdentity {
    User(UserId),
    Chat(ChatId),
}

impl From<&Message> for BindingIdentity {
    fn from(value: &Message) -> Self {
        if let Some(user) = &value.from {
            BindingIdentity::User(user.id)
        } else {
            BindingIdentity::Chat(value.chat.id)
        }
    }
}

impl Display for BindingIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingIdentity::User(id) => write!(f, "user:{}", id),
            BindingIdentity::Chat(id) => write!(f, "chat:{}", id),
        }
    }
}

impl<'a> From<&BindingEntity<'a>> for BindingIdentity {
    fn from(value: &BindingEntity) -> Self {
        match value {
            BindingEntity::User(user) => BindingIdentity::User(user.id),
            BindingEntity::Chat(chat) => BindingIdentity::Chat(chat.id),
        }
    }
}

impl<'a, 'b> TryFrom<&'b Update> for BindingEntity<'a>
where
    'b: 'a,
{
    type Error = anyhow::Error;

    fn try_from(value: &'b Update) -> Result<Self, Self::Error> {
        if let Some(user) = value.from() {
            Ok(BindingEntity::User(user))
        } else if let Some(chat) = value.chat() {
            Ok(BindingEntity::Chat(chat))
        } else {
            Err(anyhow::anyhow!("No user or chat found in the update: {value:?}"))
        }
    }
}

