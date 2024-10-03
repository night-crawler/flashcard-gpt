use crate::command::ext::CommandExt;
use crate::state::bot_state::BotState;
use crate::state::state_fields::StateFields;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use teloxide::macros::BotCommands;
use teloxide::types::InlineKeyboardButton;

#[derive(BotCommands, Clone, AsRefStr, EnumIter, EnumString, Debug)]
#[command(rename_rule = "lowercase")]
pub enum AnswerCommand {
    /// Show Article
    Article,
    /// Skip
    Skip,
    /// Next card in the Card Group
    Next,
    /// Cancel answering
    Cancel,
}

impl CommandExt for AnswerCommand {
    fn get_menu_items() -> impl Iterator<Item = InlineKeyboardButton> {
        AnswerCommand::iter().map(|cmd| InlineKeyboardButton::callback(cmd.as_ref(), cmd.as_ref()))
    }

    fn get_menu_name() -> &'static str {
        "Answering"
    }

    fn get_corresponding_state() -> BotState {
        BotState::Answering(StateFields::Answer {
            deck_card_group_id: None,
            deck_card_group_card_seq: None,
            deck_card_id: None,
            difficulty: None,
        })
    }
}
