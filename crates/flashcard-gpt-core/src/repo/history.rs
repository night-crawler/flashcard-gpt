use crate::model::history::{CreateHistory, HistoryRecord};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use std::sync::Arc;

use crate::repo::generic_repo::GenericRepo;
use crate::single_object_query;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type HistoryRepo = GenericRepo<CreateHistory, HistoryRecord, ()>;

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

    pub async fn create_custom(&self, dto: CreateHistory) -> Result<HistoryRecord, CoreError> {
        let query = format!(
            r#"
            {begin}
            
            let $id = (create history content {{
                user: $dto.user,
                deck_card: $dto.deck_card,
                deck_card_group: $dto.deck_card_group,
                difficulty: $dto.difficulty,
                hide_for: <option<duration>> $dto.hide_for,
                time: {{
                    created_at: <datetime> ($dto.time.created_at or time::now()),
                    updated_at: <datetime> ($dto.time.updated_at or time::now()),
                    hide_till: if $dto.hide_for = none {{
                        none
                    }} else {{
                        time::now() + $dto.hide_for
                    }}
                }}
            }})[0].id;
            
            select * {additional_query} from $id fetch {fetch};
            {commit}
            "#,
            begin = self.begin_transaction_statement(),
            commit = self.commit_transaction_statement(),
            fetch = self.fetch,
            additional_query = self.additional_query
        );

        single_object_query!(self.db, &query, ("dto", dto))
    }
}
