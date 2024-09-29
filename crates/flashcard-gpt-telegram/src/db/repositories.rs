use chrono_tz::Tz;
use crate::ext::binding::{BindingEntity, BindingExt};
use crate::ext::menu_repr::IteratorMenuReprExt;
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::dto::global_settings::{CreateGlobalSettingsDto, GlobalSettingsDto};
use flashcard_gpt_core::error::CoreError;
use flashcard_gpt_core::reexports::db::engine::remote::ws::Client;
use flashcard_gpt_core::reexports::db::sql::Thing;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::repo::binding::BindingRepo;
use flashcard_gpt_core::repo::card::CardRepo;
use flashcard_gpt_core::repo::card_group::CardGroupRepo;
use flashcard_gpt_core::repo::deck::DeckRepo;
use flashcard_gpt_core::repo::global_settings::GlobalSettingsRepo;
use flashcard_gpt_core::repo::tag::TagRepo;
use flashcard_gpt_core::repo::user::UserRepo;
use teloxide::types::InlineKeyboardMarkup;
use tracing::{error, Span};

#[derive(Debug, Clone)]
pub struct Repositories {
    pub tags: TagRepo,
    pub decks: DeckRepo,
    pub users: UserRepo,
    pub cards: CardRepo,
    pub card_groups: CardGroupRepo,
    pub bindings: BindingRepo,
    pub global_settings: GlobalSettingsRepo,
}

impl Repositories {
    pub fn new(db: Surreal<Client>, span: Span) -> Self {
        Self {
            tags: TagRepo::new_tag(db.clone(), span.clone(), true),
            decks: DeckRepo::new_deck(db.clone(), span.clone(), true),
            users: UserRepo::new_user(db.clone(), span.clone(), true),
            cards: CardRepo::new_card(db.clone(), span.clone(), true),
            card_groups: CardGroupRepo::new_card_group(db.clone(), span.clone(), true),
            bindings: BindingRepo::new_binding(db.clone(), span.clone(), true),
            global_settings: GlobalSettingsRepo::new_global_settings(db, span, true),
        }
    }

    pub async fn build_tag_menu(&self, user_id: Thing) -> Result<InlineKeyboardMarkup, CoreError> {
        Ok(self
            .tags
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .into_menu_repr())
    }

    pub async fn build_deck_menu(&self, user_id: Thing) -> Result<InlineKeyboardMarkup, CoreError> {
        Ok(self
            .decks
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .into_menu_repr())
    }

    pub async fn get_binding(
        &self,
        msg: impl Into<BindingEntity<'_>>,
    ) -> Result<BindingDto, CoreError> {
        self.bindings
            .get_or_create_telegram_binding(msg.into())
            .await
    }

    // TODO: implement transaction for getting an instance of global_settings
    pub async fn get_global_settings_or_default(
        &self,
        user: impl Into<Thing>,
    ) -> Result<GlobalSettingsDto, CoreError> {
        let user = user.into();
        let global_settings = match self.global_settings.get_by_user_id(user.clone()).await {
            Ok(global_settings) => global_settings,
            Err(err) => {
                error!(?err, %user, "Failed to get user settings, attempting to create default");
                self.global_settings
                    .create_custom(CreateGlobalSettingsDto {
                        user: user.clone(),
                        daily_limit: 50,
                        timetable: vec![
                            [chrono::Duration::hours(10), chrono::Duration::hours(23)],
                            // [chrono::Duration::hours(13), chrono::Duration::hours(14)],
                            // [chrono::Duration::hours(17), chrono::Duration::hours(18)],
                        ],
                        timezone: Tz::Europe__Dublin
                    })
                    .await?
            }
        };
        
        Ok(global_settings)
    }
}
