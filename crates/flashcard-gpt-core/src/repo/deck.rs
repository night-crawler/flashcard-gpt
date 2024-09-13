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
    use crate::dto::deck::Settings;
    use crate::dto::tag::CreateTagDto;
    use crate::repo::tag::TagRepo;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use std::sync::Arc;
    use testresult::TestResult;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = DeckRepo::new_deck(db.clone(), span!(Level::INFO, "deck_create"), false);
        let user = create_user("deck_create").await?;

        let tag = TagRepo::new_tag(db, span!(Level::INFO, "tag_create"), false)
            .create(CreateTagDto {
                name: Arc::from("name"),
                slug: Arc::from("slug"),
                user: user.id.clone(),
            })
            .await?;

        let deck = repo
            .create(CreateDeckDto {
                description: Some(Arc::from("description")),
                parent: None,
                user: user.id.clone(),
                title: Arc::from("title"),
                tags: vec![tag.id.clone()],
                settings: None,
            })
            .await?;

        let deck = repo.get_by_id(deck.id.clone()).await?;

        assert_eq!(deck.description.as_deref(), Some("description"));
        assert!(deck.parent.is_none());

        let deck2 = repo
            .create(CreateDeckDto {
                description: Some(Arc::from("description2")),
                parent: Some(deck.id.clone()),
                user: user.id,
                title: Arc::from("title2"),
                tags: vec![tag.id.clone()],
                settings: Some(Settings { daily_limit: 200 }),
            })
            .await?;

        assert_eq!(deck2.parent.as_ref().unwrap(), &deck.id);

        Ok(())
    }
}
