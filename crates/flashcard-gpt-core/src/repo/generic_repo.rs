use crate::error::CoreError;
use crate::ext::db::DbExt;
use crate::ext::record_id::RecordIdExt;
use std::fmt::Debug;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Field;
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

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?create_dto))]
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
    pub async fn get_by_id(&self, id: impl RecordIdExt + Debug) -> Result<Read, CoreError> {
        self.db
            .get_entity_by_id(id)
            .instrument(Span::current())
            .await
    }
}
