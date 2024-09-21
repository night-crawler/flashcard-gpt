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
    
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{TestDbExt, TEST_DB};
    use std::sync::Arc;
    use serde_json::json;
    use testresult::TestResult;
    use tracing::{span, Level};
    use crate::tests::utils::{create_card, create_tag, create_user};

    #[tokio::test]
    async fn test_create() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = CardGroupRepo::new_card_group(db, span!(Level::INFO, "card_group_create"), true);
        let user = create_user("card_group_create").await?;
        let tag = create_tag()
            .user(&user)
            .name("card_group_create")
            .slug("card_group_create")
            .call()
            .await?;
        
        let card1 = create_card().user(&user).title("card1").tags([&tag]).call().await?;
        let card2 = create_card().user(&user).title("card2").tags([&tag]).call().await?;

        let card_group = CreateCardGroupDto {
            user: user.id,
            title: Arc::from("title"),
            importance: 1,
            tags: vec![tag.id],
            cards: vec![card1.id, card2.id],
            difficulty: 2,
            data: Some(Arc::from(json!({
                "a": "b"
            }))),
        };

        let card_group = repo.create(card_group).await?;
        assert_eq!(card_group.title.as_ref(), "title");
        assert_eq!(card_group.cards.len(), 2);
        assert_eq!(card_group.tags.len(), 1);
        
        Ok(())
    }
}
