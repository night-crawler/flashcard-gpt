use crate::dto::global_settings::{CreateGlobalSettingsDto, GlobalSettingsDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type GlobalSettingsRepo = GenericRepo<CreateGlobalSettingsDto, GlobalSettingsDto, ()>;

impl GlobalSettingsRepo {
    pub fn new_global_settings(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(
            db,
            span,
            "global_settings",
            "",
            "user",
            enable_transactions,
        )
    }

    // duplicate create method with custom serializer in the query
    // I think it's identical to https://github.com/surrealdb/surrealdb/issues/3550
}
