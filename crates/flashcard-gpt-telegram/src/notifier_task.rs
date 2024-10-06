use crate::chat_manager::ChatManager;
use crate::command::answer::AnswerCommand;
use crate::db::repositories::Repositories;
use crate::ext::binding::ChatIdExt;
use crate::ext::markdown::MarkdownFormatter;
use crate::state::bot_state::{BotState, FlashGptDialogue};
use chrono::{ Timelike, Utc};
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use std::sync::Arc;
use std::time::Duration;
use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::{ApiError, Bot, RequestError};
use tokio::time::sleep;
use tracing::{debug, warn, Span};

pub async fn init_notifier(
    bot: DefaultParseMode<Bot>,
    generator: CardGeneratorService,
    storage: Arc<InMemStorage<BotState>>,
    formatter: MarkdownFormatter,
    repositories: Repositories,
    span: Span,
) -> anyhow::Result<()> {
    loop {
        let now = Utc::now();
        let bindings = repositories.bindings.list_all_not_banned().await?;
        debug!(bindings = bindings.len(), "Bindings");

        for binding in bindings {
            let generator = generator.clone();
            let binding = Arc::new(binding);
            let user = binding.user.as_ref();
            let global_settings = repositories.get_global_settings_or_default(user).await?;
            let now = now.with_timezone(&global_settings.timezone);
            if !global_settings.ts_matches(now) {
                debug!(?now, ?global_settings.timetable, %user, "Outside operating range");
                continue;
            }

            debug!(%user, global_settings.daily_limit, "Allowed daily limit for user");

            let chat_id = binding.get_chat_id()?;
            let dialogue = FlashGptDialogue::new(storage.clone(), chat_id);

            let manager = ChatManager {
                repositories: repositories.clone(),
                generator,
                formatter: formatter.clone(),
                binding: binding.clone(),
                bot: bot.clone(),
                dialogue,
                message: None,
                span: span.clone(),
            };

            let state = manager.get_state().await?;
            if !state.is_interruptible() {
                debug!(%user, %chat_id, "Non-interruptible state");
                continue;
            }

            let answered = match answer(&manager).await {
                Ok(answered) => answered,
                Err(err) => {
                    if let Some(RequestError::Api(ApiError::BotBlocked)) = err.downcast_ref::<RequestError>() {
                        warn!(%user, "Bot blocked by user");
                        manager.repositories.bindings.set_banned(binding.id.clone()).await?;
                    }
                    false
                }
            };
          
            if !answered {
                debug!(%user, %chat_id, "No active cards or card groups");
                continue;
            }
            
        }

        sleep(Duration::from_secs(10)).await;
    }
}

async fn answer(manager: &ChatManager) -> anyhow::Result<bool> {
    let now = Utc::now();
    
    let answered = if now.second() % 2 == 0 {
        manager.answer_with_card_group().await? || manager.answer_with_card().await?
    } else {
        manager.answer_with_card().await? || manager.answer_with_card_group().await?
    };

    manager.send_answer_menu().await?;
    manager.set_my_commands::<AnswerCommand>().await?;
    
    Ok(answered)
}

