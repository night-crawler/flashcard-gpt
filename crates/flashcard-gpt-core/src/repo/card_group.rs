use crate::dto::card_group::{CardGroupDto, CreateCardGroupDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type CardGroupRepo = GenericRepo<CreateCardGroupDto, CardGroupDto, ()>;

impl CardGroupRepo {
    pub fn new_card_group(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "card_group", "user, tags, cards, cards.user, cards.tags", enable_transactions)
    }
}
    