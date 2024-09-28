use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::single_object_query;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::debug;

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
        R: DeserializeOwned;
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
        R: DeserializeOwned,
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
        let fetch = if fetch.is_empty() {
            String::new()
        } else {
            format!("fetch {}", fetch)
        };
        let query = format!(
            r#"{begin}
            $id = (create type::table($table) content $dto return id)[0].id;
            return select * from type::table($table) where id=$id {fetch};
            {commit}"#
        );

        debug!(?dto, %table, %query, "Creating entity");

        single_object_query!(self, &query, ("table", table), ("dto", dto.clone()))
    }
}
