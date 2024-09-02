use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Query failed: {0:?}")]
    DbError(#[from] surrealdb::Error),
    #[error("Failed to create user: {0:?}")]
    CreateUserError(Arc<String>),
}
