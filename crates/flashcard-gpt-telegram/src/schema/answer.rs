use crate::chat_manager::ChatManager;
use crate::command::{AnswerCommand, RootCommand};
use crate::ext::card::ExtractValueExt;
use crate::state::State;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Update};
use tracing::info;

pub fn answering_schema(
) -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let answering_command_handler = teloxide::filter_command::<AnswerCommand, _>().branch(
        case![State::Answering(fields)]
            .branch(case![AnswerCommand::Article].endpoint(handle_show_article))
            .branch(case![AnswerCommand::Skip].endpoint(handle_skip_answer))
            .branch(case![AnswerCommand::Next].endpoint(handle_show_next_card))
            .branch(case![AnswerCommand::Cancel].endpoint(handle_cancel_answer)),
    );

    let answering_message_handler = Update::filter_message()
        .branch(answering_command_handler)
        .branch(case![State::Answering(fields)].endpoint(handle_answering_message));

    answering_message_handler
}

pub async fn handle_show_article(manager: ChatManager) -> anyhow::Result<()> {
    info!("handle_article");
    let fields = manager.get_state().await?.into_fields();

    if let Some(Some(dcg_id)) = fields.deck_card_group_id()
        && let dcg = manager
            .repositories
            .decks
            .get_deck_card_group(dcg_id.clone())
            .await?
        && let Some(article) = dcg.card_group.extract_str("article")
    {
        manager.send_markdown_message(article).await?;
        if let Some(code) = dcg.card_group.extract_str("commented_code") {
            manager.send_markdown_message(code).await?;
        }
    }

    manager.send_answer_menu().await?;

    Ok(())
}

pub async fn handle_commit_answer(manager: ChatManager, difficulty: u8) -> anyhow::Result<()> {
    manager.commit_answer(difficulty).await?;
    manager.set_menu_state::<RootCommand>().await?;
    Ok(())
}

pub async fn handle_skip_answer(manager: ChatManager) -> anyhow::Result<()> {
    manager.set_menu_state::<RootCommand>().await
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
        .repositories
        .decks
        .get_deck_card_group(dcg_id.clone())
        .await?;

    let Some(card) = deck_card_group.card_group.cards.get(seq) else {
        manager.send_message("No next card").await?;
        manager.send_answer_menu().await?;
        return Ok(());
    };

    manager.update_state(State::Answering(fields)).await?;
    manager.send_card(card).await?;
    manager.send_answer_menu().await?;

    Ok(())
}

pub async fn handle_cancel_answer(manager: ChatManager) -> anyhow::Result<()> {
    manager.set_menu_state::<RootCommand>().await?;
    Ok(())
}

async fn handle_answering_message(manager: ChatManager) -> anyhow::Result<()> {
    info!(?manager, "Answering message");
    Ok(())
}
