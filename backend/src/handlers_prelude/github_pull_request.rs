//! Github Pull Request are sent here.
use axum::{extract::State, http::StatusCode, Json, Router};
use axum::routing::post;
use tracing::{error, info};
use serde_json::json;
use crate::gh::create_pull_request;
use crate::AppState;
use color_eyre::Result;

#[derive(serde::Deserialize, Debug)]
pub struct CreatePRRequest {
    pub head_branch: String,    // The branch containing changes
    pub base_branch: String,     // The branch to merge into
    pub title: String,
    pub description: String,
}

pub async fn create_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreatePRRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    info!("Received create pull request request: {:?}", payload);
    let head_branch = payload.head_branch;
    let base_branch = payload.base_branch;
    let title = payload.title;
    let description = payload.description;

    // Get the access token
    let token = state.gh_credentials.get(&state.reqwest_client).await.map_err(|err| {
        error!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get access token"})))
    })?;

    // Call the function to create the pull request
    create_pull_request(
        &state.reqwest_client, 
        &token, 
        head_branch.as_str(), 
        base_branch.as_str(), 
        title.as_str(), 
        description.as_str()
    ).await.map_err(|err| {
        error!("Error creating pull request: {:?}", err);
        let error_message = format!("Failed to create pull request: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": error_message})))
    })?;

    info!("Pull request created successfully from {head_branch} to {base_branch}");
    Ok((StatusCode::CREATED, Json(json!({"message": "Pull request created successfully"}))))
}

pub async fn create_github_pull_request_route() -> Router<AppState> {
    Router::new().route("/pulls", post(create_pull_request_handler))
}
