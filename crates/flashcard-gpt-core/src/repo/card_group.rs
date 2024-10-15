use crate::model::card_group::{CardGroup, CreateCardGroup, UpdateCardGroup};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;
pub type CardGroupRepo = GenericRepo<CreateCardGroup, CardGroup, UpdateCardGroup>;

impl CardGroupRepo {
    pub fn new_card_group(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(
            db,
            span,
            "card_group",
            "",
            "user, tags, cards, cards.user, cards.tags",
            enable_transactions,
        )
    }
}
