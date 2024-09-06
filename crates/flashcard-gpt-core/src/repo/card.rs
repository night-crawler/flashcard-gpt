use crate::dto::card::{Card, CreateCardDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type CardRepo = GenericRepo<CreateCardDto, Card, ()>;

impl CardRepo {
    pub fn new_card(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "card", "user, tags", enable_transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use serde_json::json;
    use std::sync::Arc;
    use testresult::TestResult;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = CardRepo::new_card(db, span!(Level::INFO, "card_create"), true);
        let user = create_user("card_create").await?;

        let card = CreateCardDto {
            user: user.id,
            title: Arc::from("title"),
            front: Some(Arc::from("a")),
            back: Some(Arc::from("b")),
            data: Some(json!({
                "a": "b"
            })),
            hints: vec![Arc::from("a")],
            difficulty: 3,
            importance: 2,
            tags: Default::default(),
        };

        let card = repo.create(card).await?;
        assert!(card.data.is_some());

        Ok(())
    }
}
