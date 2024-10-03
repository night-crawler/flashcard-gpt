use crate::chat_manager::ChatManager;
use crate::command::answer::AnswerCommand;
use crate::db::repositories::Repositories;
use crate::ext::binding::ChatIdExt;
use crate::ext::markdown::MarkdownFormatter;
use crate::state::bot_state::{BotState, FlashGptDialogue};
use crate::state::state_fields::StateFields;
use chrono::{TimeDelta, Timelike, Utc};
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use rand::Rng;
use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;
use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::Bot;
use tokio::time::sleep;
use tracing::{debug, Span};

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
        let past_3h = now.sub(TimeDelta::hours(3));
        let bindings = repositories.bindings.list_all().await?;
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

            if now.second() % 2 == 0 {
                let mut dcgs = repositories
                    .decks
                    .get_top_ranked_card_groups(user, past_3h.to_utc())
                    .await?;
                if dcgs.is_empty() {
                    continue;
                }
                let card_id = rand::thread_rng().gen_range(0..dcgs.len());
                let dcg = dcgs.swap_remove(card_id);
                manager
                    .update_state(BotState::Answering(StateFields::Answer {
                        deck_card_group_id: Some(dcg.id),
                        deck_card_group_card_seq: Some(0),
                        deck_card_id: None,
                        difficulty: None,
                    }))
                    .await?;
                manager.send_card_group(dcg.card_group.as_ref()).await?;
                manager.send_card(dcg.card_group.cards[0].as_ref()).await?;
            } else {
                let mut dcs = repositories
                    .decks
                    .get_top_ranked_cards(user, past_3h.to_utc())
                    .await?;
                if dcs.is_empty() {
                    continue;
                }
                let id = rand::thread_rng().gen_range(0..dcs.len());
                let dc = dcs.swap_remove(id);
                manager
                    .update_state(BotState::Answering(StateFields::Answer {
                        deck_card_group_id: None,
                        deck_card_group_card_seq: None,
                        deck_card_id: Some(dc.id),
                        difficulty: None,
                    }))
                    .await?;
                manager.send_card(dc.card.as_ref()).await?
            }

            manager.send_answer_menu().await?;
            manager.set_my_commands::<AnswerCommand>().await?;
        }

        sleep(Duration::from_secs(10)).await;
    }
}
