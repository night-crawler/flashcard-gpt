use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;
use crate::command::ext::CommandExt;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum UserCommand {
    /// Edit username
    EditUsername,

    /// Edit email
    EditEmail,

    /// Edit password
    EditPassword,

    /// Cancel the current operation
    Cancel,
}

impl CommandExt for UserCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        UserCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "User Menu"
    }

    fn get_corresponding_state() -> BotState {
        BotState::InsideUserMenu(StateFields::Empty)
    }
}
