use crate::dto::history::{CreateHistoryDto, HistoryDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use std::sync::Arc;

use crate::repo::generic_repo::GenericRepo;
use crate::single_object_query;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type HistoryRepo = GenericRepo<CreateHistoryDto, HistoryDto, ()>;

impl HistoryRepo {
    pub fn new_history(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(
            db,
            span,
            "history",
            "",
            r#"
                deck_card_group, 
                deck_card_group.in, deck_card_group.out,
                deck_card_group.in.tags, 
                deck_card_group.in.user, 
                deck_card_group.out, deck_card_group.out.tags, deck_card_group.out.user, deck_card_group.out.cards,
                deck_card_group.out.cards.tags, deck_card_group.out.cards.user,
                
                deck_card,
                deck_card.in, deck_card.out,
                deck_card.in.tags, deck_card.in.user, deck_card.out.tags, deck_card.out.user
                  "#,
            enable_transactions,
        )
    }

    pub async fn create_custom(&self, dto: CreateHistoryDto) -> Result<HistoryDto, CoreError> {
        let query = format!(
            r#"
            {begin}
            
            let $id = (create history content {{
                user: $dto.user,
                deck_card: $dto.deck_card,
                deck_card_group: $dto.deck_card_group,
                difficulty: $dto.difficulty,
                time: {{
                    created_at: <datetime> ($dto.time.created_at or time::now()),
                    updated_at: <datetime> ($dto.time.updated_at or time::now()),
                }}
            }})[0].id;
            
            select * from $id fetch {fetch};
            {commit}
            "#,
            begin = self.begin_transaction_statement(),
            commit = self.commit_transaction_statement(),
            fetch = self.fetch
        );

        single_object_query!(self.db, &query, ("dto", dto))
    }
}
