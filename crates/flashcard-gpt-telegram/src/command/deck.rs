use crate::command::ext::CommandExt;
use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum DeckCommand {
    /// Show all decks
    List,

    /// Create a new deck
    Create,

    /// Continue to the next state
    Next,

    /// Cancel the current operation
    Cancel,
}

impl CommandExt for DeckCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        DeckCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Deck Menu"
    }

    fn get_corresponding_state() -> BotState {
        BotState::InsideDeckMenu(StateFields::Empty)
    }
}
