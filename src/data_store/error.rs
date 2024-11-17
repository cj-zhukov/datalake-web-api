use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("Incorrect query")]
    IncorrectQuery,

    #[error("Query result is empty")]
    QueryResultIsEmpty,
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}