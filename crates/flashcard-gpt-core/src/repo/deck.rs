use crate::dto::deck::{CreateDeckDto, DeckDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type DeckRepo = GenericRepo<CreateDeckDto, DeckDto, ()>;

impl DeckRepo {
    pub fn new_deck(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "deck", "user, tags", enable_transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use std::sync::Arc;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = DeckRepo::new_deck(db, span!(Level::INFO, "deck_create"), false);
        let user = create_user("deck_create").await?;

        let deck = CreateDeckDto {
            description: Some(Arc::from("description")),
            parent: None,
            user: user.id,
            title: Arc::from("title"),
            tags: Default::default(),
            settings: None,
        };

        let card = repo.create(deck).await?;
        println!("{:?}", card);
        Ok(())
    }
}
