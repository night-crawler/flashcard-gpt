use crate::command::ext::CommandExt;
use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum RootCommand {
    /// Display this text.
    Help,
    /// Start the purchase procedure.
    Start,
    /// Cancel the purchase procedure.
    Cancel,
    /// Edit users
    User,
    /// Edit decks
    Deck,
    /// Edit cards
    Card,
    /// Edit tags
    Tag,
    /// Edit card groups
    CardGroup,
}

impl CommandExt for RootCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        RootCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Menu"
    }

    fn get_corresponding_state() -> BotState {
        BotState::InsideRootMenu(StateFields::Empty)
    }
}
