use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

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


pub type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;
