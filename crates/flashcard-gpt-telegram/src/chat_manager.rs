use crate::command::CommandExt;
use crate::db::repositories::Repositories;
use crate::ext::dialogue::DialogueExt;
use crate::ext::menu_repr::IteratorMenuReprExt;
use crate::state::FlashGptDialogue;
use crate::state::{State, StateDescription};
use flashcard_gpt_core::dto::binding::BindingDto;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::adaptors::DefaultParseMode;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::{warn, Span};

#[derive(Debug, Clone)]
pub struct ChatManager {
    pub repositories: Repositories,
    pub binding: Arc<BindingDto>,
    pub bot: DefaultParseMode<Bot>,
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

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
        text = ?text,
    ))]
    pub async fn send_message(&self, text: impl Into<String> + Debug) -> anyhow::Result<()> {
        self.bot.send_message(self.dialogue.chat_id(), text).await?;
        Ok(())
    }

    pub async fn get_state(&self) -> anyhow::Result<State> {
        Ok(self.dialogue.get_or_default().await?)
    }

    pub async fn get_description(&self) -> anyhow::Result<StateDescription> {
        Ok(self
            .get_state()
            .await?
            .get_state_description(self.message.as_deref()))
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
    ))]
    pub async fn send_invalid_input(&self) -> anyhow::Result<()> {
        let desc = self
            .dialogue
            .get_state_description(self.message.as_deref())
            .await?;

        self.send_message(desc.invalid_input.clone().as_ref())
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

    pub fn parse_comma_separated_values(&self) -> Option<impl Iterator<Item = Arc<str>> + use<'_>> {
        if let Some(message) = self.message.as_deref()
            && let Some(text) = message.text()
        {
            return Some(
                text.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(Arc::from),
            );
        }

        None
    }

    pub fn parse_text(&self) -> Option<Arc<str>> {
        self.message
            .as_deref()
            .and_then(|message| message.text().map(Arc::from))
    }

    pub fn parse_integer<T>(&self) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        match self
            .message
            .as_deref()
            .and_then(|message| message.text().map(|text| text.parse::<T>()))?
        {
            Ok(result) => Some(result),
            Err(err) => {
                warn!(?err, "Failed to parse integer");
                None
            }
        }
    }

    pub async fn send_help<T>(&self) -> anyhow::Result<()>
    where
        T: BotCommands,
    {
        self.send_message(T::descriptions().to_string()).await?;
        Ok(())
    }

    pub async fn send_menu<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu = T::get_menu_items().into_menu_repr();
        self.bot
            .send_message(self.dialogue.chat_id(), T::get_menu_name())
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    pub async fn delete_current_message(&self) -> anyhow::Result<()> {
        if let Some(message) = self.message.as_deref() {
            self.bot
                .delete_message(self.dialogue.chat_id(), message.id)
                .await?;
        }
        Ok(())
    }

    pub async fn set_menu_state<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        self.dialogue.update(T::get_corresponding_state()).await?;
        Ok(())
    }

    pub async fn set_my_commands<T>(&self) -> anyhow::Result<()>
    where
        T: BotCommands,
    {
        self.bot.set_my_commands(T::bot_commands()).await?;
        Ok(())
    }
}
