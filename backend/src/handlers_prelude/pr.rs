use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
struct CreatePullRequestRequest {
    commit_message: String,
    branch_name: String,
    pr_title: String,
    base_branch: String,
}

#[axum::debug_handler]
pub async fn create_pull_request_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePullRequestRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let commit_message = payload.commit_message;
    let branch_name = payload.branch_name;
    let pr_title = payload.pr_title;
    let base_branch = payload.base_branch;

    let result = state.git.git_pull_request(
        &pr_title,
        &commit_message,
        &base_branch,
        &branch_name,
    ).await;

    match result {
        Ok(_) => Ok((StatusCode::CREATED, "Pull request created".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))),
    }
}
