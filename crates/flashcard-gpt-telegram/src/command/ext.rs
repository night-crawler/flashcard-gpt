use crate::state::bot_state::BotState;
use teloxide::types::InlineKeyboardButton;

pub trait CommandExt {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton>;
    fn get_menu_name() -> &'static str;
    fn get_corresponding_state() -> BotState;
    fn get_icon(&self) -> &'static str {
        ""
    }
}
