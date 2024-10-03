use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;
use crate::command::ext::CommandExt;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum TagCommand {
    /// Show all tags
    List,
}

impl CommandExt for TagCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        TagCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Tag Menu"
    }

    fn get_corresponding_state() -> BotState {
        BotState::InsideTagMenu(StateFields::Empty)
    }
}
