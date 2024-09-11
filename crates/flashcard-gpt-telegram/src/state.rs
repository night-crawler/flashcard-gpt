use crate::ext::message::MessageExt;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::types::Message;

pub type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    InsideRootMenu,
    InsideUserMenu,
    InsideDeckMenu,
    InsideCardMenu,
    InsideCardGroupMenu,
    InsideTagMenu,

    ReceiveDeckTitle,
    ReceiveDeckTags {
        title: String,
        tags: Vec<String>,
    },
    ReceiveDeckDescription {
        title: String,
        tags: Vec<String>,
    },
    ReceiveDeckParent {
        title: String,
        tags: Vec<String>,
        description: String,
    },
    ReceiveDeckSettings {
        title: String,
        tags: Vec<String>,
        description: String,
        parent: Option<String>,
    },
    ReceiveDeckConfirm {
        title: String,
        tags: Vec<String>,
        description: String,
        parent: Option<String>,
        daily_limit: Option<usize>,
    },
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
            
            State::ReceiveDeckTitle => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid deck title `{text}`")),
                    repr: Arc::from("Title: None"),
                    prompt: Arc::from("Please, enter deck title"),
                }
            },
            State::ReceiveDeckTags { title, tags } => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid tags `{text}`")),
                    repr: Arc::from(format!("Title: {title}\nTags: {tags:?}")),
                    prompt: Arc::from("Please, enter deck tags"),
                }
            },
            State::ReceiveDeckDescription { title, tags } => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid description `{text}`")),
                    repr: Arc::from(format!("Title: {title}\nTags: {tags:?}")),
                    prompt: Arc::from("Please, enter deck description"),
                }
            },
            State::ReceiveDeckParent { title, tags, description } => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid parent `{text}`")),
                    repr: Arc::from(format!("Title: {title}\nTags: {tags:?}\nDescription: {description}")),
                    prompt: Arc::from("Please, enter parent deck"),
                }
            },
            State::ReceiveDeckSettings { title, tags, description, parent } => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid daily limit `{text}`")),
                    repr: Arc::from(format!("Title: {title}\nTags: {tags:?}\nDescription: {description}\nParent: {parent:?}")),
                    prompt: Arc::from("Please, enter daily limit"),
                }
            },
            State::ReceiveDeckConfirm {
                title,
                tags,
                description,
                parent,
                daily_limit,
            } => {
                StateDescription {
                    invalid_input: Arc::from(format!("Invalid confirmation `{text}`")),
                    repr: Arc::from(format!("Title: {title}\nTags: {tags:?}\nDescription: {description}\nParent: {parent:?}\nDaily limit: {daily_limit:?}")),
                    prompt: Arc::from("Please, confirm deck creation"),
                }
            },
        }
    }
}