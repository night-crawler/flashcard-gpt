use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;
use crate::command::ext::CommandExt;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum CardCommand {
    /// Show all cards
    List,

    /// Create a new card
    Create,

    /// Generate cards using ChatGPT and add them to the deck
    Generate,

    /// Continue to the next state
    Next,

    /// Cancel the current operation
    Cancel,
}


impl CommandExt for CardCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        CardCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Card Menu"
    }

    fn get_corresponding_state() -> BotState {
        BotState::InsideCardMenu(StateFields::Empty)
    }
}
