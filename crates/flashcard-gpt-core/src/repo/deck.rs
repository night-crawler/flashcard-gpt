use crate::dto::card::CardDto;
use crate::dto::deck::{CreateDeckDto, DeckDto};
use crate::dto::deck_card::{CreateDeckCardDto, DeckCardDto};
use crate::dto::deck_card_group::{CreateDeckCardGroupDto, DeckCardGroupDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::Span;

pub type DeckRepo = GenericRepo<CreateDeckDto, DeckDto, ()>;

impl DeckRepo {
    pub fn new_deck(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "deck", "", "user, tags", enable_transactions)
    }

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?dto))]
    pub async fn relate_card(&self, dto: CreateDeckCardDto) -> Result<DeckCardDto, CoreError> {
        let dto = Arc::new(dto);
        let query = format!(
            r#"
            {begin_transaction}
            $id = (
                relate ($dto.deck) -> deck_card -> ($dto.card)
            )[0].id;
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

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?dto))]
    pub async fn relate_card_group(
        &self,
        dto: CreateDeckCardGroupDto,
    ) -> Result<DeckCardGroupDto, CoreError> {
        let dto = Arc::new(dto);
        let query = format!(
            r#"
            {begin_transaction}
            $id = (
                relate ($dto.deck) -> deck_card_group -> ($dto.card_group) 
            )[0].id;
            return select * from $id 
                fetch in, out, in.tags, out.tags, in.user, out.user, out.cards, out.cards.user, out.cards.tags;
            {commit_transaction}
            "#,
            begin_transaction = self.begin_transaction_statement(),
            commit_transaction = self.commit_transaction_statement()
        );

        let mut response = self.db.query(query).bind(("dto", dto.clone())).await?;
        response.errors_or_ok()?;
        let deck_card: Option<DeckCardGroupDto> = response.take(response.num_statements() - 1)?;
        let deck_card =
            deck_card.ok_or_else(|| CoreError::CreateError(format!("{:?}", dto).into()))?;

        Ok(deck_card)
    }
    pub async fn list_cards(
        &self,
        user: impl Into<Thing>,
        deck: impl Into<Thing>,
    ) -> Result<Vec<CardDto>, CoreError> {
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

        let mut response = self
            .db
            .query(query)
            .bind(("user", user.into()))
            .bind(("deck", deck.into()))
            .await?;

        response.errors_or_ok()?;

        Ok(response.take(response.num_statements() - 1)?)
    }
}
