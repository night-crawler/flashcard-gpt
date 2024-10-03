use crate::chat_manager::ChatManager;
use crate::db::repositories::Repositories;
use crate::ext::StrExt;
use crate::schema::receive_next;
use crate::schema::root::cancel;

use crate::command::deck::DeckCommand;
use crate::patch_state;
use crate::state::bot_state::{BotState, FlashGptDialogue};
use crate::state::state_fields::StateFields;
use anyhow::anyhow;
use flashcard_gpt_core::dto::deck::{CreateDeckDto, DeckSettings};
use std::collections::BTreeSet;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::prelude::{DependencyMap, Message, Update};

pub fn deck_schema() -> Handler<'static, DependencyMap, anyhow::Result<()>, DpHandlerDescription> {
    let deck_command_handler = teloxide::filter_command::<DeckCommand, _>().branch(
        case![BotState::InsideDeckMenu(fields)]
            .branch(case![DeckCommand::Create].endpoint(handle_create_deck)),
    );

    let deck_message_handler = Update::filter_message()
        .branch(deck_command_handler)
        .branch(
            teloxide::filter_command::<DeckCommand, _>()
                .branch(case![DeckCommand::Cancel].endpoint(cancel)),
        )
        .branch(case![BotState::ReceiveDeckTitle(fields)].endpoint(receive_deck_title))
        .branch(
            case![BotState::ReceiveDeckTags(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_tags),
        )
        .branch(case![BotState::ReceiveDeckDescription(fields)].endpoint(receive_deck_description))
        .branch(
            case![BotState::ReceiveDeckParent(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_parent),
        )
        .branch(
            case![BotState::ReceiveDeckSettingsDailyLimit(fields)]
                .branch(
                    teloxide::filter_command::<DeckCommand, _>()
                        .branch(case![DeckCommand::Next].endpoint(receive_next)),
                )
                .endpoint(receive_deck_settings),
        )
        .branch(
            case![BotState::ReceiveDeckConfirm(fields)].branch(
                teloxide::filter_command::<DeckCommand, _>()
                    .branch(case![DeckCommand::Next].endpoint(create_deck)),
            ),
        );

    deck_message_handler
}

pub async fn handle_create_deck(manager: ChatManager) -> anyhow::Result<()> {
    manager
        .send_message(
            "You are creating a new deck.\nUse /cancel to exit and /next to skip the step.\n",
        )
        .await?;
    manager
        .update_state(BotState::ReceiveDeckTitle(StateFields::default_deck()))
        .await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn receive_deck_title(manager: ChatManager, msg: Message) -> anyhow::Result<()> {
    let Some(next_title) = msg.text().map(ToOwned::to_owned) else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Deck { title },
        |title: &mut Option<Arc<str>>| {
            title.replace(Arc::from(next_title));
        }
    );

    manager
        .update_state(BotState::ReceiveDeckTags(fields))
        .await?;
    manager.send_tag_menu().await?;

    Ok(())
}

async fn receive_deck_tags(manager: ChatManager) -> anyhow::Result<()> {
    let Some(new_tags) = manager.parse_html_values(',') else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Deck { tags },
        |tags: &mut BTreeSet<Arc<str>>| { tags.extend(new_tags) }
    );
    manager
        .update_state(BotState::ReceiveDeckTags(fields))
        .await?;
    manager.send_tag_menu().await?;
    Ok(())
}

async fn receive_deck_description(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_description) = manager.parse_html() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Deck { description },
        |description: &mut Option<Arc<str>>| { description.replace(next_description) }
    );
    manager
        .update_state(BotState::ReceiveDeckParent(fields))
        .await?;
    manager.send_deck_menu().await?;

    Ok(())
}

async fn receive_deck_parent(manager: ChatManager) -> anyhow::Result<()> {
    manager.send_invalid_input().await?;
    Ok(())
}

async fn receive_deck_settings(manager: ChatManager) -> anyhow::Result<()> {
    let Some(next_daily_limit) = manager.parse_integer::<usize>() else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let fields = patch_state!(
        manager,
        StateFields::Deck { daily_limit },
        |daily_limit: &mut Option<usize>| { daily_limit.replace(next_daily_limit) }
    );

    manager
        .update_state(BotState::ReceiveDeckConfirm(fields))
        .await?;
    manager.send_state_and_prompt().await?;
    Ok(())
}

async fn create_deck(
    manager: ChatManager,
    dialogue: FlashGptDialogue,
    repositories: Repositories,
) -> anyhow::Result<()> {
    let StateFields::Deck {
        id: _,
        title,
        tags,
        description,
        parent,
        daily_limit,
    } = manager.get_state().await?.into_fields()
    else {
        manager.send_invalid_input().await?;
        return Ok(());
    };

    let parent = if let Some(parent) = parent {
        repositories
            .decks
            .get_by_id(parent.as_thing()?)
            .await?
            .id
            .into()
    } else {
        None
    };

    let user_id = manager.binding.user.id.clone();

    let tags = repositories
        .tags
        .get_or_create_tags(user_id.clone(), tags)
        .await?
        .into_iter()
        .map(|tag| tag.id)
        .collect();

    let title = title.ok_or_else(|| anyhow!("Title was not provided"))?;

    let deck = repositories
        .decks
        .create(CreateDeckDto {
            title,
            description,
            parent,
            user: user_id,
            tags,
            settings: daily_limit.map(|limit| DeckSettings { daily_limit: limit }),
        })
        .await?;

    manager
        .send_message(format!("Created a new deck: {deck:?}"))
        .await?;

    dialogue.exit().await?;
    Ok(())
}
