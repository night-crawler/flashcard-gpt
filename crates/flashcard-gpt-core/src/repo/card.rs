use crate::dto::card::{CardDto, CreateCardDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type CardRepo = GenericRepo<CreateCardDto, CardDto, ()>;

impl CardRepo {
    pub fn new_card(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "card", "user, tags", enable_transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::{create_card, create_tag, create_user};
    use crate::tests::{TestDbExt, TEST_DB};
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
            data: Some(Arc::from(json!({
                "a": "b"
            }))),
            hints: vec![Arc::from("a")],
            difficulty: 3,
            importance: 2,
            tags: Default::default(),
        };

        let card = repo.create(card).await?;
        assert!(card.data.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_deserialize_after_tag_deletion() -> TestResult {
        let user = create_user("deserialize_after_tag_deletion").await?;
        let tag = create_tag()
            .user(&user)
            .name("deserialize_after_tag_deletion")
            .slug("deserialize_after_tag_deletion")
            .call()
            .await?;

        let card = create_card()
            .tags([&tag])
            .title("deserialize_after_tag_deletion")
            .user(&user)
            .call()
            .await?;

        let repo = CardRepo::new_card(
            TEST_DB.get_client().await?,
            span!(Level::INFO, "deserialize_after_tag_deletion"),
            true,
        );
        
        let card = repo.get_by_id(card.id).await?;
        assert_eq!(card.tags.len(), 1);
        
        let tag_repo = crate::repo::tag::TagRepo::new_tag(
            TEST_DB.get_client().await?,
            span!(Level::INFO, "deserialize_after_tag_deletion"),
            true,
        );
        
        tag_repo.delete(tag.id).await?;
        
        let card = repo.get_by_id(card.id).await?;
        assert!(card.tags.is_empty());
        
        Ok(())
    }
}
