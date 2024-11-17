use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("Incorrect query")]
    IncorrectQuery,

    #[error("Query result not found")]
    QueryResultNotFound,
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}