use crate::error::CoreError;
use crate::ext::db::DbExt;
use crate::ext::response_ext::ResponseExt;
use crate::{multi_object_query, single_object_query};
use std::fmt::Debug;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::{Instrument, Span};

#[derive(Debug)]
pub struct GenericRepo<Create, Read, Update> {
    pub(super) db: Surreal<Client>,
    pub(super) span: Span,
    pub(super) table_name: &'static str,
    pub(super) additional_query: &'static str,
    pub(super) enable_transactions: bool,
    pub(super) fetch: &'static str,

    _create_phantom: std::marker::PhantomData<Create>,
    _read_phantom: std::marker::PhantomData<Read>,
    _update_phantom: std::marker::PhantomData<Update>,
}

impl<Create, Read, Update> Clone for GenericRepo<Create, Read, Update> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            span: self.span.clone(),
            table_name: self.table_name,
            additional_query: self.additional_query,
            enable_transactions: self.enable_transactions,
            fetch: self.fetch,
            _create_phantom: std::marker::PhantomData,
            _read_phantom: std::marker::PhantomData,
            _update_phantom: std::marker::PhantomData,
        }
    }
}

impl<Create, Read, Update> GenericRepo<Create, Read, Update>
where
    Create: serde::Serialize + Debug + 'static,
    Read: serde::de::DeserializeOwned,
    Update: serde::Serialize + Debug + 'static,
{
    pub fn new(
        db: Surreal<Client>,
        span: Span,
        table_name: &'static str,
        additional_query: &'static str,
        fetch: &'static str,
        enable_transactions: bool,
    ) -> Self {
        Self {
            db,
            span,
            table_name,
            additional_query,
            enable_transactions,
            fetch,
            _create_phantom: std::marker::PhantomData,
            _read_phantom: std::marker::PhantomData,
            _update_phantom: std::marker::PhantomData,
        }
    }

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?create_dto)
    )]
    pub async fn create(&self, create_dto: Create) -> Result<Read, CoreError> {
        self.db
            .create_entity(
                self.table_name,
                create_dto,
                self.fetch,
                self.enable_transactions,
            )
            .instrument(Span::current())
            .await
    }

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?id))]
    pub async fn get_by_id(&self, id: impl Into<Thing> + Debug) -> Result<Read, CoreError> {
        let query = format!(
            r#"
            select * {additional_query} from $id {fetch};
            "#,
            additional_query = self.additional_query,
            fetch = self.fetch_statement()
        );

        single_object_query!(self.db, &query, ("id", id.into()))
    }

    pub async fn list_by_user_id(&self, id: impl Into<Thing>) -> Result<Vec<Read>, CoreError> {
        let query = format!(
            r#"
            select * {additional_query} from {table_name} where user=$user_id {fetch};
            "#,
            table_name = self.table_name,
            fetch = self.fetch_statement(),
            additional_query = self.additional_query
        );

        multi_object_query!(self.db, &query, ("user_id", id.into()))
    }

    pub async fn get_by_user_id(&self, id: impl Into<Thing>) -> Result<Read, CoreError> {
        let query = format!(
            r#"
            select * {additional_query} from {table_name} where user=$user_id {fetch};
            "#,
            table_name = self.table_name,
            fetch = self.fetch_statement(),
            additional_query = self.additional_query
        );

        single_object_query!(self.db, &query, ("user_id", id.into()))
    }

    pub async fn delete(&self, id: impl Into<Thing>) -> Result<(), CoreError> {
        let query = format!(
            r#"
            delete from {table_name} where id=$id;
            "#,
            table_name = self.table_name
        );

        let mut response = self.db.query(query).bind(("id", id.into())).await?;

        response.errors_or_ok()?;

        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Read>, CoreError> {
        let query = format!(
            r#"
            select * {additional_query} from {table_name} {fetch}
            "#,
            table_name = self.table_name,
            fetch = self.fetch_statement(),
            additional_query = self.additional_query
        );

        multi_object_query!(self.db, &query,)
    }

    pub fn begin_transaction_statement(&self) -> &'static str {
        if self.enable_transactions {
            "begin transaction;"
        } else {
            ""
        }
    }

    pub fn commit_transaction_statement(&self) -> &'static str {
        if self.enable_transactions {
            "commit transaction;"
        } else {
            ""
        }
    }

    pub fn fetch_statement(&self) -> String {
        if self.fetch.is_empty() {
            String::new()
        } else {
            format!("fetch {}", self.fetch)
        }
    }
}
