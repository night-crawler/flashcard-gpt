use crate::dto::card::{Card, CreateCardDto};
use crate::error::CoreError;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct CardRepo {
    db: Surreal<Client>,
}

impl CardRepo {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn create(&self, card_dto: CreateCardDto) -> Result<Card, CoreError> {
        let card_dto = Arc::new(card_dto);
        let mut response = self.db.query(r#"
        begin transaction;
        $id = (create card content $card return id)[0].id;
        return select * from card where id=$id fetch user, tags;
        commit transaction;
        "#).bind(("card", card_dto.clone())).await?;

        let errors = response.take_errors();
        if !errors.is_empty() {
            return Err(CoreError::CreateError("card", format!("{:?}", errors).into()));
        }

        let card: Option<Card> = response.take(0)?;
        let card = card.ok_or_else(|| CoreError::CreateError("card", format!("{:?}", card_dto).into()))?;
        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = CardRepo::new(db);
        let user = create_user("card_create").await?;

        let card = CreateCardDto {
            user: user.id.unwrap(),
            title: Arc::new("title".to_string()),
            front: Some(Arc::new("a".to_string())),
            back: Some(Arc::new("b".to_string())),
            data: None,
            hints: vec![Arc::new("a".to_string())],
            difficulty: 3,
            importance: 2,
            time: None,
            tags: Default::default(),
        };

        let card = repo.create(card).await?;
        println!("{:?}", card);
        Ok(())
    }
}
