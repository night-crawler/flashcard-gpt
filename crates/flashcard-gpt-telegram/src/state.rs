use crate::ext::rendering::{DisplayJoinOrDash, OptionDisplayExt};
use crate::message_render::RenderMessageTextHelper;
use flashcard_gpt_core::reexports::db::sql::Thing;
use flashcard_gpt_core::reexports::json::Value;
use paste::paste;
use std::collections::BTreeSet;
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
    #[strum(props(name = "Card Deck"))]
    ReceiveCardDeck(StateFields),
    #[strum(props(name = "Card Creation Confirmation (/next)"))]
    ReceiveCardConfirm(StateFields),

    #[strum(props(name = "a deck that will be used for the card generation"))]
    ReceiveGenerateCardDeck(StateFields),

    #[strum(props(name = "Card Prompt"))]
    ReceiveGenerateCardPrompt(StateFields),

    #[strum(props(name = "Confirm card generation"))]
    ReceiveGenerateCardConfirm(StateFields),

    #[strum(props(name = "Answering"))]
    Answering(StateFields),
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
        let text = msg.and_then(|msg| msg.html_text()).unwrap_or_default();
        let name = self.get_str("name").unwrap_or(self.as_ref());
        let current_state_name = self.as_ref();
        let invalid_input = Arc::from(format!("Invalid <code>{name}</code>:\n{text}\n\nCurrent state: <code>{current_state_name}</code>"));
        let prompt = Arc::from(format!("Please, enter <code>{name}</code>:"));
        let repr = Arc::from(self.as_fields().to_string());

        StateDescription {
            invalid_input,
            repr,
            prompt,
        }
    }

    pub fn is_interruptible(&self) -> bool {
        match self {
            State::InsideRootMenu(_) => true,
            State::InsideUserMenu(_) => false,
            State::InsideDeckMenu(_) => false,
            State::InsideCardMenu(_) => false,
            State::InsideCardGroupMenu(_) => false,
            State::InsideTagMenu(_) => false,
            State::ReceiveDeckTitle(_) => false,
            State::ReceiveDeckTags(_) => false,
            State::ReceiveDeckDescription(_) => false,
            State::ReceiveDeckParent(_) => false,
            State::ReceiveDeckSettingsDailyLimit(_) => false,
            State::ReceiveDeckConfirm(_) => false,
            State::ReceiveCardTitle(_) => false,
            State::ReceiveCardFront(_) => false,
            State::ReceiveCardBack(_) => false,
            State::ReceiveCardHints(_) => false,
            State::ReceiveCardDifficulty(_) => false,
            State::ReceiveCardImportance(_) => false,
            State::ReceiveCardTags(_) => false,
            State::ReceiveCardDeck(_) => false,
            State::ReceiveCardConfirm(_) => false,
            State::ReceiveGenerateCardDeck(_) => false,
            State::ReceiveGenerateCardPrompt(_) => false,
            State::ReceiveGenerateCardConfirm(_) => false,
            State::Answering(_) => false,
        }
    }
}
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

macro_rules! state_variants {
    ($($variant:path),*) => {
        paste! {
            impl State {
                pub fn as_fields_mut(&mut self) -> &mut StateFields {
                    match self {
                        $(
                            State::$variant(fields) => fields,
                        )*
                    }
                }

                pub fn as_fields(&self) -> &StateFields {
                    match self {
                        $(
                            State::$variant(fields) => fields,
                        )*
                    }
                }

                pub fn into_fields(self) -> StateFields {
                    match self {
                        $(
                            State::$variant(fields) => fields,
                        )*
                    }
                }
            }
        }
    }
}

state_variants! {
    InsideRootMenu,
    InsideUserMenu,
    InsideDeckMenu,
    InsideCardMenu,
    InsideCardGroupMenu,
    InsideTagMenu,
    ReceiveDeckTitle,
    ReceiveDeckTags,
    ReceiveDeckDescription,
    ReceiveDeckParent,
    ReceiveDeckSettingsDailyLimit,
    ReceiveDeckConfirm,
    ReceiveCardTitle,
    ReceiveCardFront,
    ReceiveCardBack,
    ReceiveCardHints,
    ReceiveCardDifficulty,
    ReceiveCardImportance,
    ReceiveCardTags,
    ReceiveCardConfirm,
    ReceiveCardDeck,
    ReceiveGenerateCardDeck,
    ReceiveGenerateCardPrompt,
    ReceiveGenerateCardConfirm,
    Answering
}
