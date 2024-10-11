//! Handler for GitHub pull request creation

use axum::{extract::State, http::StatusCode, Json, Router};
use axum::routing::post;
use tracing::{error, info};
use serde_json::json;
use crate::gh::create_pull_request;
use crate::AppState;
use color_eyre::Result;
use std::convert::Infallible;

/// Struct to represent the incoming request body for creating a pull request
#[derive(serde::Deserialize)]
pub struct CreatePRRequest {
    pub head_branch: String,    // The branch containing changes
    pub base_branch: String,     // The branch to merge into
    pub title: String,
    pub description: String,
}

/// Handler for creating a pull request
pub async fn create_pull_request_handler(
    State(state): State<AppState>, // Extract AppState
    Json(payload): Json<CreatePRRequest>, // Deserialize incoming JSON payload
) -> Result<(StatusCode, Json<serde_json::Value>), Infallible> { // Use Infallible for error handling
    // Extract necessary information from the payload
    let head_branch = payload.head_branch;
    let base_branch = payload.base_branch;
    let title = payload.title;
    let description = payload.description;

    // Get the access token
    let token = match state.gh_credentials.get(&state.reqwest_client).await {
        Ok(token) => token,
        Err(err) => {
            error!("Failed to get GitHub token: {:?}", err);
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get access token"}))));
        }
    };

    // Call the function to create the pull request
    match create_pull_request(
        &state.reqwest_client, 
        &token, 
        head_branch.as_str(), 
        base_branch.as_str(), 
        title.as_str(), 
        description.as_str()
    ).await {
        Ok(_) => {
            info!("Pull request created successfully from {head_branch} to {base_branch}");
            Ok((StatusCode::CREATED, Json(json!({"message": "Pull request created successfully"}))))
        },
        Err(err) => {
            error!("Error creating pull request: {:?}", err);
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create pull request"}))))
        }
    }
}


/// Function to create GitHub routes
pub async fn create_github_pull_request_route() -> Router<AppState> {
    Router::new().route("/pulls", post(create_pull_request_handler))
}
