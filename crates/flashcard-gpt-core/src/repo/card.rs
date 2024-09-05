use crate::dto::card::{Card, CreateCardDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct CardRepo {
    db: Surreal<Client>,
    span: tracing::Span,
}

impl CardRepo {
    pub fn new(db: Surreal<Client>, span: tracing::Span) -> Self {
        Self { db, span }
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(?card_dto))]
    pub async fn create(&self, card_dto: CreateCardDto) -> Result<Card, CoreError> {
        let card_dto = Arc::new(card_dto);
        let mut response = self
            .db
            .query(
                r#"
        begin transaction;
        $id = (create card content $card return id)[0].id;
        return select * from card where id=$id fetch user, tags;
        commit transaction;
        "#,
            )
            .bind(("card", card_dto.clone()))
            .await?;

        response.errors_or_ok()?;

        let card: Option<Card> = response.take(0)?;
        let card = card.ok_or_else(|| CoreError::CreateError(format!("{:?}", card_dto).into()))?;
        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use serde_json::json;
    use std::sync::Arc;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = CardRepo::new(db, span!(Level::INFO, "card_create"));
        let user = create_user("card_create").await?;

        let card = CreateCardDto {
            user: user.id.unwrap(),
            title: Arc::new("title".to_string()),
            front: Some(Arc::new("a".to_string())),
            back: Some(Arc::new("b".to_string())),
            data: Some(json!({
                "a": "b"
            })),
            hints: vec![Arc::new("a".to_string())],
            difficulty: 3,
            importance: 2,
            tags: Default::default(),
        };

        let card = repo.create(card).await?;
        assert!(card.data.is_some());
        println!("{:?}", card);
        Ok(())
    }
}
