use crate::chat_manager::ChatManager;
use crate::db::repositories::Repositories;
use crate::ext::binding::BindingEntity;
use crate::ext::markdown::MarkdownFormatter;
use crate::schema::answer::answering_schema;
use crate::schema::card::card_schema;
use crate::schema::deck::deck_schema;
use crate::schema::root::{receive_inline_query, receive_root_menu_item, root_schema};
use crate::state::bot_state::{BotState, FlashGptDialogue};
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use std::sync::Arc;
use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateFilterExt, UpdateHandler};
use teloxide::prelude::Update;
use teloxide::types::UpdateKind;
use teloxide::{dptree, Bot};
use tracing::{debug, warn, Span};

mod answer;
mod card;
mod deck;
mod root;

pub fn schema() -> UpdateHandler<anyhow::Error> {
    let root_menu_handler = Update::filter_callback_query().endpoint(receive_root_menu_item);
    let inline_query_handler =
        Update::filter_inline_query().branch(dptree::endpoint(receive_inline_query));

    let main_branch = dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>()
        .filter_map_async(create_binding)
        .map(init_chat_manager)
        .branch(card_schema())
        .branch(deck_schema())
        .branch(root_schema())
        .branch(answering_schema())
        .branch(root_menu_handler)
        .branch(Update::filter_message().branch(dptree::endpoint(invalid_state)));

    dptree::entry()
        .inspect(|update: Update| debug!(?update, "Received update"))
        .branch(inline_query_handler)
        .branch(main_branch)
}

fn init_chat_manager(
    update: Update,
    generator: CardGeneratorService,
    repositories: Repositories,
    binding: Arc<BindingDto>,
    bot: DefaultParseMode<Bot>,
    dialogue: FlashGptDialogue,
    markdown_formatter: MarkdownFormatter,
    span: Span,
) -> ChatManager {
    let message = match update.kind {
        UpdateKind::Message(msg) => Some(Arc::new(msg)),
        _ => None,
    };

    ChatManager {
        repositories,
        binding,
        bot,
        dialogue,
        message,
        span,
        formatter: markdown_formatter,
        generator,
    }
}

async fn create_binding(update: Update, repositories: Repositories) -> Option<Arc<BindingDto>> {
    let Ok(entity) = BindingEntity::try_from(&update) else {
        warn!(?update, "Unable to create binding entity from the update.");
        return None;
    };

    let Ok(binding) = repositories.get_binding(entity.clone()).await else {
        warn!(?entity, "Unable to get binding from the repository.");
        return None;
    };
    Some(binding.into())
}

async fn receive_next(manager: ChatManager) -> anyhow::Result<()> {
    match manager.get_state().await? {
        BotState::ReceiveDeckTags(fields) => {
            let next_state = BotState::ReceiveDeckDescription(fields);
            manager.update_state(next_state).await?;
            manager.send_state_and_prompt().await?;
        }
        BotState::ReceiveCardTags(fields) => {
            let next_state = BotState::ReceiveCardDeck(fields);
            manager.update_state(next_state).await?;
            manager.send_deck_menu().await?;
        }
        BotState::ReceiveDeckDescription { .. } => {}
        BotState::ReceiveDeckParent(fields) => {
            let next_state = BotState::ReceiveDeckSettingsDailyLimit(fields);
            manager.update_state(next_state).await?;
            manager.send_state_and_prompt().await?;
        }
        BotState::ReceiveDeckSettingsDailyLimit(fields) => {
            let next_state = BotState::ReceiveDeckConfirm(fields);
            manager.update_state(next_state).await?;
            manager.send_state_and_prompt().await?;
        }
        _ => {
            manager.send_invalid_input().await?;
        }
    }

    Ok(())
}

pub async fn invalid_state(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_invalid_input().await?;
    Ok(())
}
