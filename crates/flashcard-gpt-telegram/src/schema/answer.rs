use crate::chat_manager::ChatManager;
use crate::command::answer::AnswerCommand;
use crate::command::root::RootCommand;
use crate::schema::root::handle_show_generic_menu;
use crate::state::bot_state::BotState;
use anyhow::bail;
use flashcard_gpt_core::model::card::UpdateCard;
use flashcard_gpt_core::model::card_group::UpdateCardGroup;
use flashcard_gpt_core::reexports::db::syn;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Update};
use tracing::info;

pub fn answering_schema(
) -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let answering_command_handler = teloxide::filter_command::<AnswerCommand, _>().branch(
        case![BotState::Answering(fields)]
            .branch(case![AnswerCommand::Article].endpoint(handle_show_article))
            .branch(case![AnswerCommand::Skip].endpoint(handle_skip_answer))
            .branch(case![AnswerCommand::Next].endpoint(handle_show_next_card))
            .branch(case![AnswerCommand::Hide(hide_time)].endpoint(handle_hide_card))
            .branch(case![AnswerCommand::Difficulty(difficulty)].endpoint(handle_set_difficulty))
            .branch(case![AnswerCommand::Importance(importance)].endpoint(handle_set_importance))
            .branch(case![AnswerCommand::Cancel].endpoint(handle_cancel_answer)),
    );

    let answering_message_handler = Update::filter_message()
        .branch(answering_command_handler)
        .branch(case![BotState::Answering(fields)].endpoint(handle_answering_message));

    answering_message_handler
}

async fn handle_hide_card(manager: ChatManager, duration: String) -> anyhow::Result<()> {
    let duration = match syn::duration(&duration) {
        Ok(duration) => duration,
        Err(err) => {
            manager
                .send_message(format!("Failed to parse duration `{duration}`: {err}"))
                .await?;
            return Ok(());
        }
    };

    manager.commit_answer(0, Some(duration)).await?;
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

async fn handle_set_difficulty(manager: ChatManager, difficulty: u8) -> anyhow::Result<()> {
    let fields = manager.get_state().await?.into_fields();
    if let Some(Some(dcg_id)) = fields.deck_card_group_id() {
        manager
            .update_deck_card_group_inner(
                dcg_id.clone(),
                UpdateCardGroup::builder().difficulty(difficulty).build(),
            )
            .await?;
    }

    if let Some(Some(dc_id)) = fields.deck_card_id() {
        manager
            .update_deck_card_inner(
                dc_id.clone(),
                UpdateCard::builder().difficulty(difficulty).build(),
            )
            .await?;
    }

    Ok(())
}

async fn handle_set_importance(manager: ChatManager, importance: u8) -> anyhow::Result<()> {
    let fields = manager.get_state().await?.into_fields();
    if let Some(Some(dcg_id)) = fields.deck_card_group_id() {
        manager
            .update_deck_card_group_inner(
                dcg_id.clone(),
                UpdateCardGroup::builder().importance(importance).build(),
            )
            .await?;
    }

    if let Some(Some(dc_id)) = fields.deck_card_id() {
        manager
            .update_deck_card_inner(
                dc_id.clone(),
                UpdateCard::builder().importance(importance).build(),
            )
            .await?;
    }

    Ok(())
}

pub async fn handle_show_article(manager: ChatManager) -> anyhow::Result<()> {
    let state = manager.get_state().await?;
    let fields = state.as_fields();

    let Some(Some(pk)) = fields
        .deck_card_group_id()
        .or_else(|| fields.deck_card_id())
    else {
        bail!("State has no active deck card / card group: {state:?}");
    };

    manager.send_card_group_data_by_key(pk, "article").await?;
    manager
        .send_card_group_data_by_key(pk, "commented_code")
        .await?;
    manager.send_answer_menu().await?;

    Ok(())
}

pub async fn handle_commit_answer(manager: ChatManager, difficulty: u8) -> anyhow::Result<()> {
    manager.commit_answer(difficulty, None).await?;
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

pub async fn handle_skip_answer(manager: ChatManager) -> anyhow::Result<()> {
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

pub async fn handle_show_next_card(manager: ChatManager) -> anyhow::Result<()> {
    let mut fields = manager.get_state().await?.into_fields();
    let Some(Some(seq)) = fields.deck_card_group_card_seq_mut() else {
        manager
            .send_message("No next card for card-only response")
            .await?;
        manager.send_answer_menu().await?;
        return Ok(());
    };
    *seq += 1;
    let seq = *seq;

    let Some(Some(dcg_id)) = fields.deck_card_group_id() else {
        manager.send_message("No deck ID found in state").await?;
        manager.send_answer_menu().await?;
        return Ok(());
    };

    let deck_card_group = manager
        .repo
        .decks
        .get_deck_card_group(dcg_id.clone())
        .await?;

    let Some(card) = deck_card_group.card_group.cards.get(seq) else {
        manager.send_message("No next card").await?;
        manager.send_answer_menu().await?;
        return Ok(());
    };

    manager.update_state(BotState::Answering(fields)).await?;
    manager.send_card(card).await?;
    manager.send_answer_menu().await?;

    Ok(())
}

pub async fn handle_cancel_answer(manager: ChatManager) -> anyhow::Result<()> {
    handle_show_generic_menu::<RootCommand>(manager).await?;
    Ok(())
}

async fn handle_answering_message(manager: ChatManager) -> anyhow::Result<()> {
    info!(?manager, "Answering message");
    Ok(())
}
