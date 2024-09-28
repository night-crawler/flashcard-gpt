use crate::dto::card::CardDto;
use crate::dto::deck::{CreateDeckDto, DeckDto};
use crate::dto::deck_card::{CreateDeckCardDto, DeckCardDto};
use crate::dto::deck_card_group::{CreateDeckCardGroupDto, DeckCardGroupDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use crate::{multi_object_query, single_object_query};
use chrono::Utc;
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

        single_object_query!(self.db, &query, ("dto", dto))
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

        single_object_query!(self.db, &query, ("dto", dto))
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

    pub async fn get_top_ranked_card_group(
        &self,
        user: impl Into<Thing>,
        since: chrono::DateTime<Utc>,
    ) -> Result<Vec<DeckCardGroupDto>, CoreError> {
        let query = r#"
        select 
            *,
            fn::deck_card_group_answered_times(id, <datetime> $since) as used,
            fn::rank(
                out.importance, 
                out.difficulty, 
                fn::trend(out.id).slope, 
                fn::since_last(out.id)
            ) as rank
            from deck_card_group
            where 
                out.user = $user and
                in.settings.daily_limit > fn::deck_card_group_answered_times(id, <datetime> $since)
            order by rank desc
            limit 1
            fetch 
                in, out,
                in.user, in.tags, out.user, out.cards, out.tags,
                out.cards.tags, out.cards.user
            parallel
        ;
        "#;

        multi_object_query!(self.db, query, ("user", user.into()), ("since", since))
    }

    pub async fn get_top_ranked_card(
        &self,
        user: impl Into<Thing>,
        since: chrono::DateTime<Utc>,
    ) -> Result<Vec<DeckCardDto>, CoreError> {
        let query = r#"
        select 
            *,
            fn::deck_card_answered_times(id, <datetime> $since) as used,
            fn::rank(
                out.importance, 
                out.difficulty, 
                fn::trend(out.id).slope, 
                fn::since_last(out.id)
            ) as rank
            from deck_card
            where 
                out.user = $user and
                in.settings.daily_limit > fn::deck_card_answered_times(id, <datetime> $since) and
                fn::appears_in_card_groups_in_this_deck(out, in) = 0
            order by rank desc
            limit 1
            fetch 
                in, out,
                in.user, in.tags, out.user, out.tags
            parallel
        ;
        "#;

        multi_object_query!(self.db, query, ("user", user.into()), ("since", since))
    }
}
