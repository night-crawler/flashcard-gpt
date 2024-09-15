use crate::ext::message::MessageExt;
use crate::ext::rendering::{MarkdownExt, OptionDisplayExt, VecDisplayExt};
use flashcard_gpt_core::reexports::json::Value;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;
use strum::EnumProperty as _;
use strum_macros::{AsRefStr, EnumProperty};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::types::Message;

pub type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone, Debug, EnumProperty, AsRefStr)]
pub enum State {
    #[strum(props(name = "Root Menu"))]
    InsideRootMenu(StateFields),
    #[strum(props(name = "User Menu"))]
    InsideUserMenu(StateFields),
    #[strum(props(name = "Deck Menu"))]
    InsideDeckMenu(StateFields),
    #[strum(props(name = "Card Menu"))]
    InsideCardMenu(StateFields),
    #[strum(props(name = "Card Group Menu"))]
    InsideCardGroupMenu(StateFields),
    #[strum(props(name = "Tag Menu"))]
    InsideTagMenu(StateFields),

    #[strum(props(name = "Deck Title"))]
    ReceiveDeckTitle(StateFields),
    #[strum(props(name = "Deck Tags"))]
    ReceiveDeckTags(StateFields),
    #[strum(props(name = "Deck Description"))]
    ReceiveDeckDescription(StateFields),
    #[strum(props(name = "Deck Parent"))]
    ReceiveDeckParent(StateFields),
    #[strum(props(name = "Deck Settings / Daily Limit"))]
    ReceiveDeckSettingsDailyLimit(StateFields),
    #[strum(props(name = "Deck Creation Confirmation (/next)"))]
    ReceiveDeckConfirm(StateFields),

    #[strum(props(name = "Card Title"))]
    ReceiveCardTitle(StateFields),
    #[strum(props(name = "Card Front"))]
    ReceiveCardFront(StateFields),
    #[strum(props(name = "Card Back"))]
    ReceiveCardBack(StateFields),
    #[strum(props(name = "Card Hints"))]
    ReceiveCardHints(StateFields),
    #[strum(props(name = "Card Settings / Difficulty"))]
    ReceiveCardDifficulty(StateFields),
    #[strum(props(name = "Card Settings / Importance"))]
    ReceiveCardImportance(StateFields),
    #[strum(props(name = "Card Tags"))]
    ReceiveCardTags(StateFields),
    #[strum(props(name = "Card Creation Confirmation (/next)"))]
    ReceiveCardConfirm(StateFields),
}

impl Default for State {
    fn default() -> Self {
        State::InsideRootMenu(StateFields::Empty)
    }
}

#[derive(Default, Debug)]
pub struct StateDescription {
    pub invalid_input: Arc<str>,
    pub repr: Arc<str>,
    pub prompt: Arc<str>,
}

impl State {
    pub fn get_state_description(&self, msg: Option<&Message>) -> StateDescription {
        let text = msg.map(|msg| msg.get_text()).unwrap_or_default();
        let name = self.get_str("name").unwrap_or(self.as_ref());
        let invalid_input = Arc::from(format!("Invalid {name}: `{text}`"));
        let prompt = Arc::from(format!("Please, enter {name}:"));
        let repr = Arc::from(self.get_fields().to_string());

        StateDescription {
            invalid_input,
            repr,
            prompt,
        }
    }

    pub fn get_fields_mut(&mut self) -> &mut StateFields {
        match self {
            State::InsideRootMenu(fields) => fields,
            State::InsideUserMenu(fields) => fields,
            State::InsideDeckMenu(fields) => fields,
            State::InsideCardMenu(fields) => fields,
            State::InsideCardGroupMenu(fields) => fields,
            State::InsideTagMenu(fields) => fields,
            State::ReceiveDeckTitle(fields) => fields,
            State::ReceiveDeckTags(fields) => fields,
            State::ReceiveDeckDescription(fields) => fields,
            State::ReceiveDeckParent(fields) => fields,
            State::ReceiveDeckSettingsDailyLimit(fields) => fields,
            State::ReceiveDeckConfirm(fields) => fields,
            State::ReceiveCardTitle(fields) => fields,
            State::ReceiveCardFront(fields) => fields,
            State::ReceiveCardBack(fields) => fields,
            State::ReceiveCardHints(fields) => fields,
            State::ReceiveCardDifficulty(fields) => fields,
            State::ReceiveCardImportance(fields) => fields,
            State::ReceiveCardTags(fields) => fields,
            State::ReceiveCardConfirm(fields) => fields,
        }
    }

    pub fn get_fields(&self) -> &StateFields {
        match self {
            State::InsideRootMenu(fields) => fields,
            State::InsideUserMenu(fields) => fields,
            State::InsideDeckMenu(fields) => fields,
            State::InsideCardMenu(fields) => fields,
            State::InsideCardGroupMenu(fields) => fields,
            State::InsideTagMenu(fields) => fields,
            State::ReceiveDeckTitle(fields) => fields,
            State::ReceiveDeckTags(fields) => fields,
            State::ReceiveDeckDescription(fields) => fields,
            State::ReceiveDeckParent(fields) => fields,
            State::ReceiveDeckSettingsDailyLimit(fields) => fields,
            State::ReceiveDeckConfirm(fields) => fields,
            State::ReceiveCardTitle(fields) => fields,
            State::ReceiveCardFront(fields) => fields,
            State::ReceiveCardBack(fields) => fields,
            State::ReceiveCardHints(fields) => fields,
            State::ReceiveCardDifficulty(fields) => fields,
            State::ReceiveCardImportance(fields) => fields,
            State::ReceiveCardTags(fields) => fields,
            State::ReceiveCardConfirm(fields) => fields,
        }
    }

    pub fn take_fields(self) -> StateFields {
        match self {
            State::InsideRootMenu(fields) => fields,
            State::InsideUserMenu(fields) => fields,
            State::InsideDeckMenu(fields) => fields,
            State::InsideCardMenu(fields) => fields,
            State::InsideCardGroupMenu(fields) => fields,
            State::InsideTagMenu(fields) => fields,
            State::ReceiveDeckTitle(fields) => fields,
            State::ReceiveDeckTags(fields) => fields,
            State::ReceiveDeckDescription(fields) => fields,
            State::ReceiveDeckParent(fields) => fields,
            State::ReceiveDeckSettingsDailyLimit(fields) => fields,
            State::ReceiveDeckConfirm(fields) => fields,
            State::ReceiveCardTitle(fields) => fields,
            State::ReceiveCardFront(fields) => fields,
            State::ReceiveCardBack(fields) => fields,
            State::ReceiveCardHints(fields) => fields,
            State::ReceiveCardDifficulty(fields) => fields,
            State::ReceiveCardImportance(fields) => fields,
            State::ReceiveCardTags(fields) => fields,
            State::ReceiveCardConfirm(fields) => fields,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum StateFields {
    #[default]
    Empty,
    Deck {
        id: Option<Arc<str>>,
        title: Option<Arc<str>>,
        tags: Vec<Arc<str>>,
        description: Option<Arc<str>>,
        parent: Option<Arc<str>>,
        daily_limit: Option<usize>,
    },

    Card {
        id: Option<Arc<str>>,
        title: Option<Arc<str>>,
        front: Option<Arc<str>>,
        back: Option<Arc<str>>,
        hints: Vec<Arc<str>>,
        difficulty: Option<u8>,
        importance: Option<u8>,
        data: Option<Arc<Value>>,
        tags: Vec<Arc<str>>,
    },
}

impl Display for StateFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateFields::Empty => write!(f, "Empty"),
            StateFields::Deck {
                id,
                title,
                tags,
                description,
                parent,
                daily_limit,
            } => {
                writeln!(f, "id: {}", id.to_string_or_dash())?;
                writeln!(f, "title: {}", title.to_string_or_dash())?;
                writeln!(f, "tags: {}", tags.join_or_dash())?;
                writeln!(f, "description: {}", description.to_string_or_dash())?;
                writeln!(f, "parent: {}", parent.to_string_or_dash())?;
                write!(f, "daily_limit: {}", daily_limit.to_string_or_dash())
            }
            StateFields::Card {
                id,
                title,
                front,
                back,
                hints,
                difficulty,
                importance,
                data,
                tags,
            } => {
                writeln!(f, "id: {}", id.to_string_or_dash())?;
                writeln!(f, "title: {}", title.to_string_or_dash())?;
                writeln!(f, "front: {}", front.to_string_or_dash())?;
                writeln!(f, "back: {}", back.to_string_or_dash())?;
                writeln!(f, "hints: {}", hints.join_or_dash())?;
                writeln!(f, "difficulty: {}", difficulty.to_string_or_dash())?;
                writeln!(f, "importance: {}", importance.to_string_or_dash())?;
                writeln!(f, "data: {}", data.to_string_or_dash())?;
                write!(f, "tags: {}", tags.join_or_dash())
            }
        }
    }
}

impl StateFields {
    pub fn default_card() -> Self {
        Self::Card {
            id: None,
            title: None,
            front: None,
            back: None,
            hints: vec![],
            difficulty: None,
            importance: None,
            data: None,
            tags: vec![],
        }
    }
    
    pub fn default_deck() -> Self {
        Self::Deck {
            id: None,
            title: None,
            tags: vec![],
            description: None,
            parent: None,
            daily_limit: None,
        }
    }
}

impl MarkdownExt for StateFields {
    fn to_markdown(&self) -> String {
        match self {
            StateFields::Empty => "Empty".to_string(),
            StateFields::Deck {
                id,
                title,
                tags,
                description,
                parent,
                daily_limit,
            } => format!(
                "**id**: {}\n\
                 **title**: {}\n\
                 **tags**: {}\n\
                 **description**: {}\n\
                 **parent**: {}\n\
                 **daily_limit**: {}",
                id.to_string_or_dash(),
                title.to_string_or_dash(),
                tags.join_or_dash(),
                description.to_string_or_dash(),
                parent.to_string_or_dash(),
                daily_limit.to_string_or_dash()
            ),
            StateFields::Card {
                id,
                title,
                front,
                back,
                hints,
                difficulty,
                importance,
                data,
                tags,
            } => format!(
                "**id**: {}\n\
                 **title**: {}\n\
                 **front**: {}\n\
                 **back**: {}\n\
                 **hints**: {}\n\
                 **difficulty**: {}\n\
                 **importance**: {}\n\
                 **data**: {}\n\
                 **tags**: {}",
                id.to_string_or_dash(),
                title.to_string_or_dash(),
                front.to_string_or_dash(),
                back.to_string_or_dash(),
                hints.join_or_dash(),
                difficulty.to_string_or_dash(),
                importance.to_string_or_dash(),
                data.to_string_or_dash(),
                tags.join_or_dash()
            ),
        }
    }
}
