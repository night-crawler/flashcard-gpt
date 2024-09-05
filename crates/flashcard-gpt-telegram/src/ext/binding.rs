use flashcard_gpt_core::dto::binding::{Binding, GetOrCreateBindingDto};
use flashcard_gpt_core::error::CoreError;
use flashcard_gpt_core::repo::binding::BindingRepo;
use std::future::Future;
use std::sync::Arc;
use teloxide::prelude::*;

pub trait BindingExt {
    fn get_or_create_telegram_binding(&self, msg: &Message) -> impl Future<Output = Result<Binding, CoreError>>;
}

impl BindingExt for BindingRepo {
    async fn get_or_create_telegram_binding(&self, msg: &Message) -> Result<Binding, CoreError> {
        let source_id = Arc::from(if let Some(user) = &msg.from {
            format!("user:{}", user.id)
        } else {
            format!("chat:{}", msg.chat.id)
        });

        let binding = self.get_binding(Arc::clone(&source_id)).await?;
        if let Some(binding) = binding {
            return Ok(binding);
        }

        let (email, data, name) = if let Some(user) = &msg.from {
            let username = user.username.clone().unwrap_or_else(|| user.id.to_string());
            let serialized = flashcard_gpt_core::reexports::json::to_value(user)?;

            (
                format!("user-{}@telegram-flash-gpt.example.com", username),
                serialized,
                user.full_name(),
            )
        } else {
            let serialized = flashcard_gpt_core::reexports::json::to_value(&msg.chat)?;
            let name = msg
                .chat
                .title()
                .or_else(|| msg.chat.username())
                .map(|name| name.to_string())
                .unwrap_or_else(|| msg.chat.id.to_string());
            (
                format!("chat-{}telegram-flash-gpt.example.com", msg.chat.id),
                serialized,
                name,
            )
        };

        let binding_dto = GetOrCreateBindingDto {
            source_id,
            name: Arc::from(name),
            type_name: Arc::from("telegram"),
            password: Arc::from(uuid::Uuid::new_v4().to_string()),
            data: Some(data),
            email: Arc::from(email),
        };

        self.get_or_create_binding(binding_dto).await
    }
}
