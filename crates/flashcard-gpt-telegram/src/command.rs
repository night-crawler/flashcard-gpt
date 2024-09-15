use crate::state::{State, StateFields};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;

pub trait CommandExt {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton>;
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

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum DeckCommand {
    /// Show all decks
    List,

    /// Create a new deck
    Create,

    /// Continue to the next state
    Next,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum UserCommand {
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
    /// Show all cards
    List,
    
    /// Create a new card
    Create,

    /// Continue to the next state
    Next,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum CardGroupCommand {
    /// Show all card groups
    List,
}

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString)]
#[command(rename_rule = "lowercase")]
pub enum TagCommand {
    /// Show all tags
    List,
}

impl CommandExt for RootCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        RootCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideRootMenu(StateFields::Empty)
    }
}

impl CommandExt for DeckCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        DeckCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Deck Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideDeckMenu(StateFields::Empty)
    }
}

impl CommandExt for UserCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        UserCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "User Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideUserMenu(StateFields::Empty)
    }
}

impl CommandExt for CardCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        CardCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Card Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideCardMenu(StateFields::Empty)
    }
}

impl CommandExt for CardGroupCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        CardGroupCommand::iter()
            .map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Card Group Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideCardGroupMenu(StateFields::Empty)
    }
}

impl CommandExt for TagCommand {
    fn get_menu_items() -> impl Iterator<Item=InlineKeyboardButton> {
        TagCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Tag Menu"
    }

    fn get_corresponding_state() -> State {
        State::InsideTagMenu(StateFields::Empty)
    }
}
