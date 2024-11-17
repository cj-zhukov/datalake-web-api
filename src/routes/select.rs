use color_eyre::Result;
use axum::{
    extract::State, 
    http::StatusCode, 
    response::IntoResponse, 
    Json
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, data_store::aws::Table};
use crate::data_store::error::DataStoreError;
use crate::error::ApiError;

#[derive(Deserialize)]
pub struct SelectRequest {
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectResponse {
    pub message: String,
    pub content: Vec<Table>,
}

pub async fn post_select(
    State(state): State<AppState>,
    Json(input): Json<SelectRequest>
) -> Result<impl IntoResponse, ApiError> {
    match Table::select(state.ctx, input.query.as_deref()).await {
        Ok(records) => {
            let res = Json(SelectResponse {
                message: "Table selected".to_string(),
                content: records,
            });
        
            Ok((StatusCode::OK, res))
        },
        Err(e) => match e {
            DataStoreError::IncorrectQuery => Err(ApiError::IncorrectQuery),
            DataStoreError::QueryResultIsEmpty => Err(ApiError::QueryResultIsEmpty),
            DataStoreError::UnexpectedError(e) => Err(ApiError::UnexpectedError(e.into())),
        }
    }
}