use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DCKError {
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Transaction execution failed: {0}")]
    TransactionExecution(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Concurrency limit exceeded or cancelled")]
    ConcurrencyLimit,

    #[error("Linear algebra failure: {0}")]
    LinAlg(String),

    #[error("Internal invariant violated: {0}")]
    Invariant(String),
}
