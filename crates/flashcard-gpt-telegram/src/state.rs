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
}


pub type FlashGptDialogue = Dialogue<State, InMemStorage<State>>;
