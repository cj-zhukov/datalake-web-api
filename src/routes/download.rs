use color_eyre::Result;
use axum::{
    body::Body, extract::{Path, State}, http::StatusCode, response::Response, Json
};
use serde::Deserialize;

use crate::{app_state::AppState, data_store::aws::Table, utils::utils::zip_data};

#[derive(Deserialize)]
pub struct QueryDownloadRequest {
    pub query: Option<String>,
}

pub async fn post_download(
    State(state): State<AppState>,
    Json(input): Json<QueryDownloadRequest>
) -> Result<Response, StatusCode> {
    let data = Table::download(state.ctx, input.query.as_deref()).await.unwrap();
    let zipped_data = zip_data(data).await.unwrap();
    let body = Body::from(zipped_data);
    let response = Response::builder()
        .status(200)
        .body(body)
        .unwrap();

    Ok(response)
}

pub async fn post_download_id(Path(file_name): Path<String>) -> Result<Response, StatusCode> {
    let data = Table::download_id(&file_name).await.unwrap();
    let body = Body::from(data);
    let response = Response::builder()
        .status(200)
        .body(body)
        .unwrap();

    Ok(response)
}