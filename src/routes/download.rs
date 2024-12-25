use color_eyre::Result;
use axum::{
    body::Body, 
    extract::State, 
    response::Response, 
    Json
};
use serde::Deserialize;

use crate::{app_state::AppState, data_store::aws::Table, utils::utils::zip_data};
use crate::data_store::error::DataStoreError;
use crate::error::ApiError;

#[derive(Deserialize)]
pub struct QueryDownloadRequest {
    pub query: Option<String>,
}

pub async fn post_download(
    State(state): State<AppState>,
    Json(input): Json<QueryDownloadRequest>
) -> Result<Response, ApiError> {
    match Table::download(state.ctx, state.client, input.query.as_deref()).await {
        Ok(records) => {
            let zipped_data = zip_data(records)
                .await
                .map_err(|e| ApiError::UnexpectedError(e.into()))?;
            
            let body = Body::from(zipped_data);

            let response = Response::builder()
                .status(200)
                .body(body)
                .map_err(|e| ApiError::UnexpectedError(e.into()))?;
    
            Ok(response)
        },
        Err(e) => match e {
            DataStoreError::IncorrectQuery => Err(ApiError::IncorrectQuery),
            DataStoreError::QueryResultIsEmpty => Err(ApiError::QueryResultIsEmpty),
            DataStoreError::UnexpectedError(e) => Err(ApiError::UnexpectedError(e.into())),
        }
    }
}