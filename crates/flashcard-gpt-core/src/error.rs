use std::sync::Arc;
use tracing_subscriber::util::TryInitError;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Query failed: {0:?}")]
    DbError(#[from] surrealdb::Error),
    #[error("Failed to create: {0}")]
    CreateError(Arc<str>),

    #[error("Not found: {0}")]
    NotFound(Arc<str>),

    #[error("Mutex is poisoned: {0}")]
    MutexPoisoned(String),

    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing_subscriber::filter::ParseError),

    #[error("Json parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Tracking init error: {0:?}")]
    TrackingInitError(#[from] TryInitError)
}
