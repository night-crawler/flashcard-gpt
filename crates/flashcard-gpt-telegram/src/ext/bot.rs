use crate::command::CommandExt;
use crate::ext::menu_repr::{IteratorMenuReprExt, MenuReprExt};
use crate::state::StateDescription;
use flashcard_gpt_core::dto::deck::DeckDto;
use std::future::Future;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::{ChatId, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

pub trait BotExt {
    fn send_menu<T>(&self, chat_id: ChatId) -> impl Future<Output = anyhow::Result<()>>
    where
        T: CommandExt;

    fn send_help<T>(&self, chat_id: ChatId) -> impl Future<Output = anyhow::Result<()>>
    where
        T: BotCommands;

    fn send_decks_menu(
        &self,
        chat_id: ChatId,
        decks: Vec<DeckDto>,
    ) -> impl Future<Output = anyhow::Result<()>>;

    fn send_invalid_input(
        &self,
        msg: &Message,
        state_description: &StateDescription,
    ) -> impl Future<Output = anyhow::Result<()>>;

    fn send_state_and_prompt(
        &self,
        msg: &Message,
        state_description: &StateDescription,
    ) -> impl Future<Output = anyhow::Result<()>>;

    fn send_state_and_prompt_with_keyboard(
        &self,
        msg: &Message,
        state_description: &StateDescription,
        keyboard: InlineKeyboardMarkup,
    ) -> impl Future<Output = anyhow::Result<()>>;
}

impl BotExt for Bot {
    async fn send_menu<T>(&self, chat_id: ChatId) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu = T::get_menu_items().into_menu_repr();
        self.send_message(chat_id, T::get_menu_name())
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    async fn send_help<T>(&self, chat_id: ChatId) -> anyhow::Result<()>
    where
        T: BotCommands,
    {
        self.send_message(chat_id, T::descriptions().to_string())
            .await?;
        Ok(())
    }

    async fn send_decks_menu(&self, chat_id: ChatId, decks: Vec<DeckDto>) -> anyhow::Result<()> {
        let menu = decks.into_iter().into_menu_repr();
        self.send_message(chat_id, "Choose a deck")
            .reply_markup(menu)
            .await?;

        Ok(())
    }

    async fn send_invalid_input(
        &self,
        msg: &Message,
        state_description: &StateDescription,
    ) -> anyhow::Result<()> {
        self.send_message(
            msg.chat.id,
            state_description.invalid_input.clone().as_ref(),
        )
        .await?;
        Ok(())
    }

    async fn send_state_and_prompt(
        &self,
        msg: &Message,
        state_description: &StateDescription,
    ) -> anyhow::Result<()> {
        let combined = format!("{}\n{}", state_description.repr, state_description.prompt);
        self.send_message(msg.chat.id, combined).await?;
        Ok(())
    }

    async fn send_state_and_prompt_with_keyboard(
        &self,
        msg: &Message,
        state_description: &StateDescription,
        keyboard: InlineKeyboardMarkup,
    ) -> anyhow::Result<()> {
        let combined = format!("{}\n{}", state_description.repr, state_description.prompt);
        self.send_message(msg.chat.id, combined)
            .reply_markup(keyboard)
            .await?;
        Ok(())
    }
}
