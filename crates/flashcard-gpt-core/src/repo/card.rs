use crate::dto::card::{CardDto, CreateCardDto, UpdateCardDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type CardRepo = GenericRepo<CreateCardDto, CardDto, UpdateCardDto>;

impl CardRepo {
    pub fn new_card(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "card", "", "user, tags", enable_transactions)
    }
}
