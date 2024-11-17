use anyhow::Result;
use axum::{
    body::Body, extract::Path, http::StatusCode, response::Response, Json
};
use serde::Deserialize;

use crate::{data_store::aws::Table, utils::utils::zip_data};

#[derive(Deserialize)]
pub struct QueryDownloadRequest {
    pub query: Option<String>,
}

pub async fn post_download(Json(input): Json<QueryDownloadRequest>) -> Result<Response, StatusCode> {
    let data = Table::download(input.query.as_deref()).await.unwrap();
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