use crate::ext::message::MessageExt;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::types::Message;

pub type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;


#[derive(Debug, Default, Clone)]
pub struct ModifyDeckFields {
    pub id: Option<Arc<str>>,
    pub title: Option<Arc<str>>,
    pub tags: Vec<Arc<str>>,
    pub description: Option<Arc<str>>,
    pub parent: Option<Arc<str>>,
    pub daily_limit: Option<usize>,
}

impl Display for ModifyDeckFields {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}\n\
            title: {}\n\
            tags: {}\n\
            description: {}\n\
            parent: {}\n\
            daily_limit: {}",
            self.id.as_deref().unwrap_or("None"),
            self.title.as_deref().unwrap_or("None"),
            self.tags.iter().map(|tag| tag.as_ref()).collect::<Vec<_>>().join(", "),
            self.description.as_deref().unwrap_or("None"),
            self.parent.as_deref().unwrap_or("None"),
            self.daily_limit.map_or("None".to_string(), |limit| limit.to_string()),
        )
    }
}

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    InsideRootMenu,
    InsideUserMenu,
    InsideDeckMenu,
    InsideCardMenu,
    InsideCardGroupMenu,
    InsideTagMenu,

    ReceiveDeckTitle(ModifyDeckFields),
    ReceiveDeckTags(ModifyDeckFields),
    ReceiveDeckDescription(ModifyDeckFields),
    ReceiveDeckParent(ModifyDeckFields),
    ReceiveDeckSettingsDailyLimit(ModifyDeckFields),
    ReceiveDeckConfirm(ModifyDeckFields),
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
        match self {
            State::InsideRootMenu => StateDescription::default(),
            State::InsideUserMenu => StateDescription::default(),
            State::InsideDeckMenu => StateDescription::default(),
            State::InsideCardMenu => StateDescription::default(),
            State::InsideCardGroupMenu => StateDescription::default(),
            State::InsideTagMenu => StateDescription::default(),

            State::ReceiveDeckTitle(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid deck title `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, enter deck title"),
                }
            }
            State::ReceiveDeckTags(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid tags `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, enter deck tags"),
                }
            }
            State::ReceiveDeckDescription(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid description `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, enter deck description"),
                }
            }
            State::ReceiveDeckParent(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid parent `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, enter parent deck"),
                }
            }
            State::ReceiveDeckSettingsDailyLimit(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid daily limit `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, enter daily limit"),
                }
            }
            State::ReceiveDeckConfirm(fields) => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid confirmation `{text}`")),
                    repr: Arc::from(fields.to_string()),
                    prompt: Arc::from("Please, confirm deck creation"),
                }
            }
        }
    }
}