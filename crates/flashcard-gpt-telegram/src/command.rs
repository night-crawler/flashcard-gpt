use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;
use crate::state::State;

pub trait CommandExt {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton>;
    fn get_menu_name() -> &'static str;
    fn get_corresponding_state() -> State;
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum RootCommand {
    /// Display this text.
    Help,
    /// Start the purchase procedure.
    Start,
    /// Cancel the purchase procedure.
    Cancel,
    /// Edit user
    User,
    /// Edit deck
    Deck,
    /// Edit card
    Card,
    /// Edit card group
    CardGroup,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum DeckCommand {
    /// Display this text
    Help,

    /// Show all decks
    List,
    
    /// Create a new deck
    Create,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum UserCommand {
    /// Display this text
    Help,

    /// Edit username
    EditUsername,

    /// Edit email
    EditEmail,

    /// Edit password
    EditPassword,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum CardCommand {
    /// Display this text
    Help,

    /// Show all cards
    List,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum CardGroupCommand {
    /// Display this text
    Help,

    /// Show all card groups
    List,
}

impl CommandExt for RootCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        RootCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideRootMenu
    }
}

impl CommandExt for DeckCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        DeckCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Deck Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideDeckMenu
    }
}

impl CommandExt for UserCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        UserCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "User Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideUserMenu
    }
}

impl CommandExt for CardCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        CardCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Card Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideCardMenu
    }
}

impl CommandExt for CardGroupCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        CardGroupCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Card Group Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideCardGroupMenu
    }
}
