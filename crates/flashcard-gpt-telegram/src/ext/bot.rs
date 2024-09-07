use crate::command::CommandExt;
use flashcard_gpt_core::dto::deck::DeckDto;
use std::future::Future;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup};
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
}

impl BotExt for Bot {
    async fn send_menu<T>(&self, chat_id: ChatId) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu_items = T::get_menu_items();
        self.send_message(chat_id, T::get_menu_name())
            .reply_markup(InlineKeyboardMarkup::new([menu_items]))
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
        let items = decks
            .into_iter()
            .map(|deck| InlineKeyboardButton::callback(deck.title.as_ref(), deck.id.to_string()));
        
        self.send_message(chat_id, "Choose a deck")
            .reply_markup(InlineKeyboardMarkup::new([items]))
            .await?;

        Ok(())
    }
}
