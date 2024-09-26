use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;
use crate::dto::history::{CreateHistoryDto, HistoryDto};
use crate::repo::generic_repo::GenericRepo;

pub type HistoryRepo = GenericRepo<CreateHistoryDto, HistoryDto, ()>;

impl HistoryRepo {
    pub fn new_history(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "history", "", "user", enable_transactions)
    }
}
