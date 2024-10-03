use crate::message_render::RenderMessageTextHelper;
use crate::state::state_description::StateDescription;
use crate::state::state_fields::StateFields;
use paste::paste;
use std::sync::Arc;
use strum::EnumProperty as _;
use strum_macros::{AsRefStr, EnumProperty};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::types::Message;

pub type FlashGptDialogue = Dialogue<BotState, InMemStorage<BotState>>;

#[derive(Clone, Debug, EnumProperty, AsRefStr)]
pub enum BotState {
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

    #[strum(props(name = "Confirm card generation (use /next)"))]
    ReceiveGenerateCardConfirm(StateFields),

    #[strum(props(name = "Answering"))]
    Answering(StateFields),
}

impl Default for BotState {
    fn default() -> Self {
        BotState::InsideRootMenu(StateFields::Empty)
    }
}

impl BotState {
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
            BotState::InsideRootMenu(_) => true,
            BotState::InsideUserMenu(_) => false,
            BotState::InsideDeckMenu(_) => false,
            BotState::InsideCardMenu(_) => false,
            BotState::InsideCardGroupMenu(_) => false,
            BotState::InsideTagMenu(_) => false,
            BotState::ReceiveDeckTitle(_) => false,
            BotState::ReceiveDeckTags(_) => false,
            BotState::ReceiveDeckDescription(_) => false,
            BotState::ReceiveDeckParent(_) => false,
            BotState::ReceiveDeckSettingsDailyLimit(_) => false,
            BotState::ReceiveDeckConfirm(_) => false,
            BotState::ReceiveCardTitle(_) => false,
            BotState::ReceiveCardFront(_) => false,
            BotState::ReceiveCardBack(_) => false,
            BotState::ReceiveCardHints(_) => false,
            BotState::ReceiveCardDifficulty(_) => false,
            BotState::ReceiveCardImportance(_) => false,
            BotState::ReceiveCardTags(_) => false,
            BotState::ReceiveCardDeck(_) => false,
            BotState::ReceiveCardConfirm(_) => false,
            BotState::ReceiveGenerateCardDeck(_) => false,
            BotState::ReceiveGenerateCardPrompt(_) => false,
            BotState::ReceiveGenerateCardConfirm(_) => false,
            BotState::Answering(_) => false,
        }
    }
}

macro_rules! state_variants {
    ($($variant:path),*) => {
        paste! {
            impl BotState {
                pub fn as_fields_mut(&mut self) -> &mut StateFields {
                    match self {
                        $(
                            Self::$variant(fields) => fields,
                        )*
                    }
                }

                pub fn as_fields(&self) -> &StateFields {
                    match self {
                        $(
                            Self::$variant(fields) => fields,
                        )*
                    }
                }

                pub fn into_fields(self) -> StateFields {
                    match self {
                        $(
                            Self::$variant(fields) => fields,
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
