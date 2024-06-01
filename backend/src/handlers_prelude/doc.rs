use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetCpuQuery {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetCpuResponse {
    pub contents: String,
}

/// This handler accepts a `GET` request to `/api/doc?path=`.
pub async fn get_doc_handler(
    State(state): State<AppState>,
    Query(query): Query<GetCpuQuery>,
) -> Result<Json<GetCpuResponse>, (StatusCode, &'static str)> {
    match state.git.get_doc(&query.path) {
        Ok(maybe_doc) => maybe_doc.map_or(
            Err((
                StatusCode::NOT_FOUND,
                "The file at the provided path was not found.",
            )),
            |doc| Ok(Json(GetCpuResponse { contents: doc })),
        ),
        Err(e) => {
            warn!(
                "Failed to fetch doc with path: {:?}; error: {:?}",
                query.path, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Fetch failed, check server logs for more info",
            ))
        }
    }
}
