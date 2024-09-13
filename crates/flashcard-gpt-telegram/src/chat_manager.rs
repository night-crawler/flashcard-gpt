use crate::state::{ModifyDeckFields, State, StateDescription};

use crate::db::repositories::Repositories;
use crate::ext::dialogue::DialogueExt;
use crate::state::FlashGptDialogue;
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::reexports::trace::Span;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::Bot;

#[derive(Debug, Clone)]
pub struct ChatManager {
    pub repositories: Repositories,
    pub binding: Arc<BindingDto>,
    pub bot: Bot,
    pub dialogue: FlashGptDialogue,
    pub message: Option<Arc<Message>>,
    pub span: Span,
}

impl ChatManager {
    pub async fn update_state(&self, next_state: State) -> anyhow::Result<StateDescription> {
        let desc = next_state.get_state_description(self.message.as_deref());
        self.dialogue.update(next_state).await?;
        Ok(desc)
    }
    
    pub async fn send_message(&self, text: impl Into<String>) -> anyhow::Result<()> {
        self.bot.send_message(self.dialogue.chat_id(), text).await?;
        Ok(())
    }

    pub async fn get_state(&self) -> anyhow::Result<State> {
        Ok(self.dialogue.get_or_default().await?)
    }

    pub async fn get_description(&self) -> anyhow::Result<StateDescription> {
        Ok(self.get_state().await?.get_state_description(self.message.as_deref()))
    }

    pub async fn send_invalid_input(&self) -> anyhow::Result<()> {
        let desc = self
            .dialogue
            .get_state_description(self.message.as_deref())
            .await?;

        self.bot
            .send_message(self.dialogue.chat_id(), desc.invalid_input.clone().as_ref())
            .await?;

        Ok(())
    }

    pub async fn send_tag_menu(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let tag_menu = self
            .repositories
            .build_tag_menu(self.binding.user.id.clone())
            .await?;

        let combined = format!("{}\n{}", desc.repr, desc.prompt);
        self.bot
            .send_message(self.dialogue.chat_id(), combined)
            .reply_markup(tag_menu)
            .await?;

        Ok(())
    }

    pub async fn send_deck_menu(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let tag_menu = self
            .repositories
            .build_deck_menu(self.binding.user.id.clone())
            .await?;

        let combined = format!("{}\n\n{}", desc.repr, desc.prompt);
        self.bot
            .send_message(self.dialogue.chat_id(), combined)
            .reply_markup(tag_menu)
            .await?;

        Ok(())
    }

    pub async fn send_state_and_prompt(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let combined = format!("{}\n\n{}", desc.repr, desc.prompt);
        self.bot
            .send_message(self.dialogue.chat_id(), combined)
            .await?;

        Ok(())
    }

    pub async fn get_modify_deck_fields(&self) -> anyhow::Result<ModifyDeckFields> {
        let state = self.get_state().await?;

        let fields = match state {
            State::ReceiveDeckTitle(fields) => fields,
            State::ReceiveDeckTags(fields) => fields,
            State::ReceiveDeckDescription(fields) => fields,
            State::ReceiveDeckParent(fields) => fields,
            State::ReceiveDeckSettingsDailyLimit(fields) => fields,
            State::ReceiveDeckConfirm(fields) => fields,
            _ => {
                anyhow::bail!(
                    "Tried to extract ModifyDeckFields from an invalid state: {:?}",
                    state
                );
            }
        };

        Ok(fields)
    }
}
