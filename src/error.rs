use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, 
};
use color_eyre::eyre::Report;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum APIError {
    #[error("Incorrect query")]
    IncorrectQuery,

    #[error("Query result not found")]
    QueryResultNotFound,
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            APIError::IncorrectQuery => (StatusCode::BAD_REQUEST, "Incorrect query"),
            APIError::QueryResultNotFound => (StatusCode::NOT_FOUND, "Not found"),
            APIError::UnexpectedError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}