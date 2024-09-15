use crate::ext::binding::{BindingEntity, BindingExt};
use crate::ext::menu_repr::IteratorMenuReprExt;
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::dto::tag::TagDto;
use flashcard_gpt_core::error::CoreError;
use flashcard_gpt_core::reexports::db::engine::remote::ws::Client;
use flashcard_gpt_core::reexports::db::sql::Thing;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::repo::binding::BindingRepo;
use flashcard_gpt_core::repo::card::CardRepo;
use flashcard_gpt_core::repo::deck::DeckRepo;
use flashcard_gpt_core::repo::tag::TagRepo;
use flashcard_gpt_core::repo::user::UserRepo;
use std::sync::Arc;
use teloxide::types::InlineKeyboardMarkup;
use tracing::Span;

#[derive(Debug, Clone)]
pub struct Repositories {
    pub tags: TagRepo,
    pub decks: DeckRepo,
    pub users: UserRepo,
    pub cards: CardRepo,
    pub bindings: BindingRepo,
}

impl Repositories {
    pub fn new(db: Surreal<Client>, span: Span) -> Self {
        Self {
            tags: TagRepo::new_tag(db.clone(), span.clone(), true),
            decks: DeckRepo::new_deck(db.clone(), span.clone(), true),
            users: UserRepo::new_user(db.clone(), span.clone(), true),
            cards: CardRepo::new_card(db.clone(), span.clone(), true),
            bindings: BindingRepo::new_binding(db, span, true),
        }
    }

    pub async fn build_tag_menu(&self, user_id: Thing) -> anyhow::Result<InlineKeyboardMarkup> {
        Ok(self
            .tags
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .into_menu_repr())
    }

    pub async fn build_deck_menu(&self, user_id: Thing) -> anyhow::Result<InlineKeyboardMarkup> {
        Ok(self
            .decks
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .into_menu_repr())
    }

    pub async fn get_or_create_tags(
        &self,
        user_id: Thing,
        tags: Vec<Arc<str>>,
    ) -> anyhow::Result<Vec<TagDto>> {
        // we assume that slug after slugify stays the same
        let tags = tags
            .into_iter()
            .map(|tag| {
                let slug = slug::slugify(&tag);
                (Arc::from(tag), Arc::from(slug))
            })
            .collect();

        Ok(self.tags.get_or_create_tags(user_id, tags).await?)
    }

    pub async fn get_binding(
        &self,
        msg: impl Into<BindingEntity<'_>>,
    ) -> Result<BindingDto, CoreError> {
        self.bindings
            .get_or_create_telegram_binding(msg.into())
            .await
    }
}
