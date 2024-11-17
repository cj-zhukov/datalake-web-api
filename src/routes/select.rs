use color_eyre::Result;
use axum::{
    extract::State, http::StatusCode, response::IntoResponse, Json
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, data_store::aws::Table};

#[derive(Deserialize)]
pub struct SelectRequest {
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectResponse {
    pub message: String,
    pub content: Option<Vec<Table>>,
}

pub async fn post_select(
    State(state): State<AppState>,
    Json(input): Json<SelectRequest>
) -> Result<impl IntoResponse, Json<SelectResponse>> {
    let records = Table::select(state.ctx, input.query.as_deref())
        .await
        .map_err(|_| {
        Json(SelectResponse {
            message: format!("Failed selecting table with query: {:?}", input.query.as_deref()),
            content: None,
        })
    })?;

    let res = Json(SelectResponse {
        message: "Table selected".to_string(),
        content: records,
    });

    Ok((StatusCode::OK, res))
}