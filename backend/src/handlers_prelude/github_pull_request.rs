//! Github Pull Request are sent here.
use axum::{extract::State, http::StatusCode, Json, Router};
use axum::routing::post;
use tracing::{error, info};
use serde::{Serialize, Deserialize};
use crate::gh::create_pull_request;
use crate::AppState;
use color_eyre::Result;

/// Request structure for creating a pull request
#[derive(Deserialize, Debug)]
pub struct CreatePRRequest {
    pub head_branch: String,
    pub base_branch: String,
    pub title: String,
    pub description: String,
}

/// Struct to represent a successful pull request creation response
#[derive(Serialize, Debug)]
pub struct CreatePRSuccessResponse {
    pub message: String,
}

/// Struct to represent an error response
#[derive(Serialize, Debug)]
pub struct CreatePRErrorResponse {
    pub error: String,
}

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,  // Optional to handle cases where there is no data (error cases)
}

#[derive(Serialize, Debug)]
pub struct CreatePRData {
    pub pull_request_url: String,
}

/// Handler to create a pull request from a specified head branch to a base branch.
///
/// # Arguments
/// * `state` - Application state containing configuration and necessary clients (e.g., GitHub credentials and HTTP client).
/// * `payload` - A `CreatePRRequest` containing the following fields:
///     * `head_branch` - The name of the branch containing the changes to be merged.
///     * `base_branch` - The name of the branch into which the changes should be merged.
///     * `title` - The title of the pull request.
///     * `description` - A description of the changes in the pull request.
///
/// # Returns
/// On success, returns `StatusCode::CREATED` and a JSON message indicating success.
/// On failure, returns an appropriate `StatusCode` and a JSON error message.
///
/// # Errors
/// * Returns `StatusCode::INTERNAL_SERVER_ERROR` if there is an issue obtaining the GitHub token or creating the pull request.
/// * Logs errors if encountered during the process and returns a corresponding error message in the response.
pub async fn create_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreatePRRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreatePRData>>), (StatusCode, Json<CreatePRErrorResponse>)> {
    info!("Received create pull request request: {:?}", payload);

    let head_branch = payload.head_branch;
    let base_branch = payload.base_branch;
    let title = payload.title;
    let description = payload.description;

    // Get the access token from the state
    let token = state.gh_credentials.get(&state.reqwest_client).await.map_err(|err| { //Cannot use warp_err here it doesn't return the correct type
        error!("Failed to get GitHub token: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreatePRErrorResponse {
                error: "Failed to get access token".to_string(),
            }),
        )
    })?;

    // Attempt to create the pull request on GitHub
    let pull_request_url = create_pull_request(
        &state.reqwest_client,
        &token,
        &head_branch,
        &base_branch,
        &title,
        &description,
    ).await.map_err(|err| {
        error!("Error creating pull request: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreatePRErrorResponse {
                error: format!("Failed to create pull request: {:?}", err),
            }),
        )
    })?; // Ensure to handle the response appropriately

    // If successful, return a success response
    info!("Pull request created successfully from {head_branch} to {base_branch}");
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            status: "success".to_string(),
            message: "Pull request created successfully".to_string(),
            data: Some(CreatePRData {
                pull_request_url, 
            }),
        }),
    ))
}

/// Route definition for the GitHub pull request creation endpoint
pub async fn create_github_pull_request_route() -> Router<AppState> {
    Router::new().route("/pulls", post(create_pull_request_handler))
}
