use crate::ext::rendering::{DisplayJoinOrDash, OptionDisplayExt};
use flashcard_gpt_core::reexports::db::sql::Thing;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Default, Clone, enum_fields::EnumFields)]
pub enum StateFields {
    #[default]
    Empty,
    Deck {
        id: Option<Arc<str>>,
        title: Option<Arc<str>>,
        tags: BTreeSet<Arc<str>>,
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
        tags: BTreeSet<Arc<str>>,
        deck: Option<Arc<str>>,
    },

    GenerateCard {
        deck: Option<Arc<str>>,
        prompt: Option<Arc<str>>,
    },

    Answer {
        deck_card_group_id: Option<Thing>,
        deck_card_group_card_seq: Option<usize>,
        deck_card_id: Option<Thing>,
        difficulty: Option<u8>,
    },
}

impl Display for StateFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateFields::Empty => write!(f, "<i>Empty</i>"),
            StateFields::Deck {
                id,
                title,
                tags,
                description,
                parent,
                daily_limit,
            } => {
                writeln!(f, "<b>id:</b> {}", id.to_string_or_dash())?;
                writeln!(f, "<b>title:</b> {}", title.to_string_or_dash())?;
                writeln!(f, "<b>tags:</b> {}", tags.join_or_dash())?;
                writeln!(f, "<b>description:</b> {}", description.to_string_or_dash())?;
                writeln!(f, "<b>parent:</b> {}", parent.to_string_or_dash())?;
                write!(f, "<b>daily_limit:</b> {}", daily_limit.to_string_or_dash())
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
                deck,
            } => {
                writeln!(f, "<b>id:</b> {}", id.to_string_or_dash())?;
                writeln!(f, "<b>title:</b> {}", title.to_string_or_dash())?;
                writeln!(f, "<b>front:</b> {}", front.to_string_or_dash())?;
                writeln!(f, "<b>back:</b> {}", back.to_string_or_dash())?;
                writeln!(f, "<b>hints:</b> {}", hints.join_or_dash())?;
                writeln!(f, "<b>difficulty:</b> {}", difficulty.to_string_or_dash())?;
                writeln!(f, "<b>importance:</b> {}", importance.to_string_or_dash())?;
                writeln!(f, "<b>data:</b> {}", data.to_string_or_dash())?;
                writeln!(f, "<b>tags:</b> {}", tags.join_or_dash())?;
                write!(f, "<b>deck:</b> {}", deck.to_string_or_dash())
            }
            StateFields::GenerateCard { deck, prompt } => {
                writeln!(f, "<b>Deck:</b> {}", deck.to_string_or_dash())?;
                write!(f, "<b>Prompt:</b> {}", prompt.to_string_or_dash())
            }
            StateFields::Answer {
                deck_card_group_id: card_group_id,
                deck_card_group_card_seq: card_group_card_seq,
                deck_card_id: card_id,
                difficulty,
            } => {
                writeln!(
                    f,
                    "<b>Card Group:</b> {} / {}",
                    card_group_id.to_string_or_dash(),
                    card_group_card_seq.to_string_or_dash()
                )?;
                writeln!(f, "<b>Card:</b> {}", card_id.to_string_or_dash())?;
                write!(f, "<b>Difficulty:</b> {}", difficulty.to_string_or_dash())
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
            tags: Default::default(),
            deck: None,
        }
    }

    pub fn default_deck() -> Self {
        Self::Deck {
            id: None,
            title: None,
            tags: Default::default(),
            description: None,
            parent: None,
            daily_limit: None,
        }
    }

    pub fn default_answer() -> Self {
        Self::Answer {
            deck_card_group_id: None,
            deck_card_group_card_seq: None,
            deck_card_id: None,
            difficulty: None,
        }
    }
}
