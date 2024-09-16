use crate::dto::deck::{CreateDeckDto, DeckDto};
use crate::dto::deck_card::{CreateDeckCardDto, DeckCardDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::Span;
use crate::dto::card::CardDto;

pub type DeckRepo = GenericRepo<CreateDeckDto, DeckDto, ()>;

impl DeckRepo {
    pub fn new_deck(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "deck", "user, tags", enable_transactions)
    }

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?dto))]
    pub async fn relate_card(&self, dto: CreateDeckCardDto) -> Result<DeckCardDto, CoreError> {
        let dto = Arc::new(dto);
        let query = format!(
            r#"
            {begin_transaction}
            $id = (relate ($dto.deck) -> deck_card -> ($dto.card) content {{
                difficulty: $dto.difficulty,
                importance: $dto.importance,
            }})[0].id;
            return select * from $id fetch in, out, in.tags, out.tags, in.user, out.user;
            {commit_transaction}
            "#,
            begin_transaction = self.begin_transaction_statement(),
            commit_transaction = self.commit_transaction_statement()
        );

        let mut response = self.db.query(query).bind(("dto", dto.clone())).await?;
        response.errors_or_ok()?;
        let deck_card: Option<DeckCardDto> = response.take(response.num_statements() - 1)?;
        let deck_card =
            deck_card.ok_or_else(|| CoreError::CreateError(format!("{:?}", dto).into()))?;

        Ok(deck_card)
    }
    
    pub async fn list_cards(&self, user: impl Into<Thing>, deck: impl Into<Thing>) -> Result<Vec<CardDto>, CoreError> {
        let query = format!(
            r#"
            {begin_transaction}
            let $results = (
                select ->deck_card->card as cards FROM $deck
                where user = $user
                fetch cards, cards.user, cards.tags
            )[0].cards;
            return select * from $results order by title;           
            {commit_transaction}
            "#,
            begin_transaction = self.begin_transaction_statement(),
            commit_transaction = self.commit_transaction_statement()
        );

        let mut response = self.db.query(query)
            .bind(("user", user.into()))
            .bind(("deck", deck.into()))
            .await?;
        
        response.errors_or_ok()?;
        
        Ok(response.take(response.num_statements() - 1)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::deck::DeckSettings;
    use crate::tests::utils::{create_card, create_deck, create_tag, create_user};
    use crate::tests::TEST_DB;
    use std::sync::Arc;
    use testresult::TestResult;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = DeckRepo::new_deck(db.clone(), span!(Level::INFO, "deck_create"), false);
        let user = create_user("deck_create").await?;
        
        let tag = create_tag()
            .user(&user)
            .name("name")
            .call()
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
        
        let deck2 = create_deck()
            .title("sample deck 2")
            .user(&user)
            .tags([&tag])
            .parent(deck.id.clone())
            .settings(DeckSettings { daily_limit: 200 })
            .call()
            .await?;

        assert_eq!(deck2.parent.as_ref().unwrap(), &deck.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_relation_ops() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = DeckRepo::new_deck(
            db.clone(),
            span!(Level::INFO, "deck_create_relation"),
            false,
        );
        let user = create_user("deck_create_relation").await?;

        let tag = create_tag()
            .user(&user)
            .name("name")
            .call()
            .await?;

        let deck1 = create_deck()
            .title("sample deck")
            .user(&user)
            .tags([&tag])
            .call()
            .await?;
        
        let deck2 = create_deck()
            .title("sample deck 2")
            .user(&user)
            .tags([&tag])
            .call()
            .await?;
        
        let mut relations = vec![];
        
        for _ in 0..10 {
            let card = create_card()
                .user(&user)
                .tags([&tag])
                .title(format!("card {}", 1))
                .call()
                .await?;
            
            let relation = repo.relate_card(CreateDeckCardDto {
                deck: deck1.id.clone(),
                card: card.id.clone(),
                importance: 0,
                difficulty: 0,
            }).await?;
            
            relations.push(relation);
        }
        
        let cards = repo.list_cards(&user, &deck1).await?;
        assert_eq!(cards.len(), 10);
        
        let cards2 = repo.list_cards(&user, &deck2).await?;
        assert!(cards2.is_empty());
        
        Ok(())
    }
}
