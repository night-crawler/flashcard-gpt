use crate::error::CoreError;
use crate::ext::record_id::RecordIdExt;
use crate::ext::response_ext::ResponseExt;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub trait DbExt {
    fn create_entity<W, R>(
        &self,
        table: &'static str,
        dto: W,
        fetch: &'static str,
        enable_transactions: bool,
    ) -> impl Future<Output = Result<R, CoreError>>
    where
        W: serde::Serialize + Debug + 'static,
        R: serde::de::DeserializeOwned;

    fn get_entity_by_id<R>(
        &self,
        id: impl RecordIdExt + Debug,
    ) -> impl Future<Output = Result<R, CoreError>>
    where
        R: serde::de::DeserializeOwned;
}

impl DbExt for Surreal<Client> {
    async fn create_entity<W, R>(
        &self,
        table: &'static str,
        dto: W,
        fetch: &'static str,
        enable_transactions: bool,
    ) -> Result<R, CoreError>
    where
        W: serde::Serialize + Debug + 'static,
        R: serde::de::DeserializeOwned,
    {
        let dto = Arc::new(dto);
        let begin = if enable_transactions {
            "begin transaction;"
        } else {
            ""
        };
        let commit = if enable_transactions {
            "commit transaction;"
        } else {
            ""
        };
        let mut response = self
            .query(format!(
                r#"
                {begin};
                $id = (create type::table($table) content $dto return id)[0].id;
                return select * from type::table($table) where id=$id fetch {fetch};
                {commit};
            "#
            ))
            .bind(("table", table))
            .bind(("dto", dto.clone()))
            .await?;

        response.errors_or_ok()?;

        let result: Option<R> = response.take(response.num_statements() - 1)?;

        let card = result.ok_or_else(|| CoreError::CreateError(format!("{:?}", dto).into()))?;
        Ok(card)
    }
    async fn get_entity_by_id<R>(&self, id: impl RecordIdExt + Debug) -> Result<R, CoreError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = id.record_id();
        let result: Option<R> = self.select(id.clone()).await?;
        if let Some(result) = result {
            Ok(result)
        } else {
            Err(CoreError::NotFound(Arc::from(id.to_string())))
        }
    }
}
