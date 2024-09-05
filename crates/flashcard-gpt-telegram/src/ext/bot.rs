use crate::command::CommandExt;
use std::future::Future;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, InlineKeyboardMarkup};
use teloxide::Bot;

pub trait BotExt {
    fn send_menu<T>(&self, chat_id: ChatId) -> impl Future<Output = anyhow::Result<()>>
    where
        T: CommandExt;
}

impl BotExt for Bot {
    async fn send_menu<T>(&self, chat_id: ChatId) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu_items = T::get_menu_items();
        self.send_message(chat_id, "Root menu:")
            .reply_markup(InlineKeyboardMarkup::new([menu_items]))
            .await?;
        Ok(())
    }
}
