use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use log::error;
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
        Ok(maybe_doc) => match maybe_doc {
            Some(doc) => return Ok(Json(GetCpuResponse { contents: doc })),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    "The file at the provided path was not found.",
                ))
            }
        },
        Err(e) => {
            error!(
                "Failed to fetch doc with path: {}; error: {:?}",
                query.path, e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Fetch failed, check server logs for more info",
            ));
        }
    }
}
