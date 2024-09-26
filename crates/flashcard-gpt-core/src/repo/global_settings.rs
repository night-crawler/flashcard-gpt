use std::future::IntoFuture;
use crate::dto::global_settings::{CreateGlobalSettingsDto, GlobalSettingsDto};
use crate::error::CoreError;
use crate::repo::generic_repo::GenericRepo;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::{Instrument, Span};
use crate::ext::response_ext::ResponseExt;

pub type GlobalSettingsRepo = GenericRepo<CreateGlobalSettingsDto, GlobalSettingsDto, ()>;

impl GlobalSettingsRepo {
    pub fn new_global_settings(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(
            db,
            span,
            "global_settings",
            ", timetable.map(|$pair| $pair.map(|$duration| <string> $duration))",
            "user",
            enable_transactions,
        )
    }

    // duplicate create method with custom serializer in the query
    pub async fn create_custom(
        &self,
        dto: CreateGlobalSettingsDto,
    ) -> Result<GlobalSettingsDto, CoreError> {
        let dto = Arc::new(dto);
        let query = format!(
            r#"
        {begin}
        let $timetable = $dto.timetable.map(|$pair| $pair.map(|$duration| <duration> $duration));
        
        $id = (create global_settings content {{
            user: $dto.user,
            daily_limit: $dto.daily_limit,
            timetable: $timetable
        }}).id;
        return select * {additional_query} from $id fetch user;
        {commit}
        "#,
            begin = self.begin_transaction_statement(),
            commit = self.commit_transaction_statement(),
            additional_query = self.additional_query
        );

        let mut response = self
            .db
            .query(query)
            .bind(("dto", dto.clone()))
            .into_future()
            .instrument(self.span.clone())
            .await?;

        response.errors_or_ok()?;

        let result: Option<GlobalSettingsDto> = response.take(response.num_statements() - 1)?;
        let result = result.ok_or_else(|| CoreError::CreateError(format!("{:?}", dto).into()))?;
        Ok(result)
    }
}
