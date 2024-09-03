use surrealdb::{RecordId, RecordIdKey};
use surrealdb::sql::Thing;
use crate::dto::user::User;

pub trait RecordIdExt {
    fn record_id(&self) -> RecordId;
}

impl RecordIdExt for User {
    fn record_id(&self) -> RecordId {
        self.id.clone().unwrap().record_id()
    }
}

impl RecordIdExt for Thing {
    fn record_id(&self) -> RecordId {
        RecordId::from_table_key(self.tb.clone(), RecordIdKey::from_inner(self.id.clone()))
    }
}

impl RecordIdExt for RecordId {
    fn record_id(&self) -> RecordId {
        self.clone()
    }
}
