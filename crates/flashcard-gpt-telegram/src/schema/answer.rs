use crate::chat_manager::ChatManager;
use crate::command::AnsweringCommand;
use crate::patch_state;
use crate::state::{State, StateFields};
use anyhow::Result;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Update};
use tracing::info;

pub fn answering_schema() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    let answering_command_handler = teloxide::filter_command::<AnsweringCommand, _>().branch(
        case![State::Answering(fields)]
            .branch(case![AnsweringCommand::Article].endpoint(handle_article))
            .branch(case![AnsweringCommand::Commit(difficulty)].endpoint(handle_commit))
            .branch(case![AnsweringCommand::Skip].endpoint(handle_skip))
            .branch(case![AnsweringCommand::Next].endpoint(handle_next_card))
            .branch(case![AnsweringCommand::Cancel].endpoint(cancel_answering)),
    );

    let answering_message_handler = Update::filter_message()
        .branch(answering_command_handler)
        .branch(case![State::Answering(fields)].endpoint(handle_answering_message));

    answering_message_handler
}

async fn handle_article(manager: ChatManager) -> Result<()> {
    info!("handling article");
    Ok(())
}

async fn handle_commit(manager: ChatManager, difficulty: u8) -> Result<()> {
    info!(?difficulty, "Committing");
    Ok(())
}

async fn handle_skip(manager: ChatManager) -> Result<()> {
    info!(?manager, "Skipping");
    handle_next_card(manager).await
}

async fn handle_next_card(manager: ChatManager) -> Result<()> {
    let fields = patch_state!(
        manager,
        StateFields::Answer { deck_card_group_card_seq },
        |seq: &mut Option<usize>| {
            seq.replace(seq.unwrap_or(0) + 1)
        }
    );
    
    let StateFields::Answer {
        deck_card_group_id: Some(dcg_id), deck_card_group_card_seq: Some(seq), ..
    } = &fields
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let deck_card_group = manager.repositories.decks.get_deck_card_group(dcg_id.clone()).await?;

    let Some(card) = deck_card_group.card_group.cards.get(*seq) else {
        manager.send_message("No next card").await?;
        return Ok(());
    };
    
    manager.update_state(State::Answering(fields)).await?;
    manager.send_card(card).await?;
    manager.send_menu::<AnsweringCommand>().await?;

    Ok(())
}

async fn cancel_answering(manager: ChatManager) -> Result<()> {
    info!(?manager, "Canceling answering");
    Ok(())
}

// Handler for unexpected messages during answering
async fn handle_answering_message(manager: ChatManager) -> Result<()> {
    info!(?manager, "Answering message");
    Ok(())
}
