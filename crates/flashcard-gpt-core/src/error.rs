use std::sync::Arc;
#[cfg(test)]

use testcontainers::TestcontainersError;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Query failed: {0:?}")]
    DbError(#[from] surrealdb::Error),
    #[error("Failed to create: {0}")]
    CreateError(Arc<str>),

    #[cfg(test)]
    #[error("Failed to start test container: {0}")]
    TestContainersError(#[from] TestcontainersError),
 
    #[error("Not found: {0}:{1}")]
    NotFound(&'static str, String),

    #[error("Mutex is poisoned: {0}")]
    MutexPoisoned(String),

    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing_subscriber::filter::ParseError)
}
