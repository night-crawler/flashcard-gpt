use crate::error::CoreError;
use crate::ext::db::DbExt;
use crate::ext::response_ext::ResponseExt;
use std::fmt::Debug;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::{Instrument, Span};

#[derive(Debug)]
pub struct GenericRepo<Create, Read, Update> {
    pub(super) db: Surreal<Client>,
    pub(super) span: Span,
    pub(super) table_name: &'static str,
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
        fetch: &'static str,
        enable_transactions: bool,
    ) -> Self {
        Self {
            db,
            span,
            table_name,
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
    pub async fn get_by_id(&self, id: Thing) -> Result<Read, CoreError> {
        self.db
            .get_entity_by_id(id, self.fetch)
            .instrument(Span::current())
            .await
    }

    pub async fn list_by_user_id(&self, user_id: Thing) -> Result<Vec<Read>, CoreError> {
        let fetch = if self.fetch.is_empty() {
            String::new()
        } else {
            format!("fetch {}", self.fetch)
        };
        let query = format!(
            r#"
            select * from {table_name} where user=$user_id {fetch};
            "#,
            table_name = self.table_name,
            fetch = fetch
        );

        let mut response = self.db.query(query).bind(("user_id", user_id)).await?;

        response.errors_or_ok()?;

        let result: Vec<Read> = response.take(0)?;
        Ok(result)
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
}
