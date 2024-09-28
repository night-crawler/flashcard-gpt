use std::sync::Arc;
use tracing_subscriber::util::TryInitError;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Query failed: {0:?}")]
    DbError(#[from] surrealdb::Error),
    
    #[error("Database error: {0}")]
    DbQueryHasErrors(Arc<str>),

    #[error("Result not found: {0}")]
    DbQueryResultNotFound(Arc<str>),

    #[error("Not found: {0}")]
    NotFound(Arc<str>),

    #[error("Mutex is poisoned: {0}")]
    MutexPoisoned(String),

    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing_subscriber::filter::ParseError),

    #[error("Json parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Tracking init error: {0:?}")]
    TrackingInitError(#[from] TryInitError),

    #[error("Format and execute error: {0}")]
    LlmFormatAndExecuteError(#[from] llm_chain::frame::FormatAndExecuteError),

    #[error("Executor error: {0}")]
    LlmExecutorError(#[from] llm_chain::traits::ExecutorError),

    #[error("No LLM steps provided: {0}")]
    LlmNoLlmStepsProvided(Arc<str>),

    #[error("First step must have exactly one input parameter, but got {0}")]
    LlmFirstStepInputParamError(Arc<str>),

    #[error("LLM Body Extract error: {0}")]
    LlmBodyExtractError(Arc<str>),

    #[error("LLM result is missing: {0}")]
    LlmResultMissing(Arc<str>),
}
