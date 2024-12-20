use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json, Router,
};
use axum::routing::{get, post, put};
use tracing::{error, info};
use serde::{Serialize, Deserialize};
use crate::gh::GitHubClient;
use crate::handlers_prelude::eyre_to_axum_err;
use crate::AppState;
use color_eyre::Result;

/// General API response structure
#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,
}

/// Error response structure
#[derive(Serialize, Debug)]
pub struct ApiErrorResponse {
    pub error: String,
}

/// Represents the structure for a pull request creation response
#[derive(Serialize, Debug)]
pub struct CreatePRData {
    pub pull_request_url: String,
}

/// Represents the structure for a branch listing response
#[derive(Serialize, Debug)]
pub struct BranchesData {
    pub branches: Vec<String>,
}

/// Request structure for creating a pull request
#[derive(Deserialize, Debug)]
pub struct CreatePRRequest {
    pub head_branch: String,
    pub base_branch: String,
    pub title: String,
    pub description: String,
}

/// Retrieves the GitHub access token from the application state.
async fn get_github_token(state: &AppState) -> Result<String, (StatusCode, String)> {
    state.gh_credentials.get(&state.reqwest_client, &state.config.oauth.github.client_id).await.map_err(|err| {
        eyre_to_axum_err(err)
    })
}

/// Fetches the list of branches from a GitHub repository.
pub async fn list_branches_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<BranchesData>>), (StatusCode, String)> {
    info!("Received request to fetch branches");

    // Get the GitHub access token
    let token = get_github_token(&state).await.map_err(|err| {
        // Format the error message as a string
        let error_message = format!("Error: {:?}", err);  // Use {:?} to format the tuple
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    // Retrieve the repository URL from state (assuming it is stored in state.config.files.repo_url)
    let repo_url = state.config.files.repo_url.clone();

    // Create an instance of GitHubClient with the repository URL, reqwest client, and token
    let github_client = GitHubClient::new(repo_url, state.reqwest_client.clone(), token);

    // Fetch the branch details from GitHub using the GitHubClient instance
    let branch_details = github_client
        .get_all_branch_details() // Call the method on the GitHubClient instance
        .await
        .map_err(|err| {
            // Handle errors in fetching branch details (e.g., connection issues)
            eyre_to_axum_err(err)
        })?;

    // Extract branch names and handle protection status if needed
    let branches: Vec<String> = branch_details
        .into_iter()
        .map(|branch| {
            let name = branch.name.clone();
            if branch.protected {
                format!("{} (protected)", name)
            } else {
                name
            }
        })
        .collect();

    // Return success response
    info!("Branches fetched successfully.");
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            status: "success".to_string(),
            message: "Branches fetched successfully".to_string(),
            data: Some(BranchesData { branches }),
        }),
    ))
}

/// Handler to create a pull request from a specified head branch to a base branch.
pub async fn create_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreatePRRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreatePRData>>), (StatusCode, String)> {
    info!("Received create pull request request: {:?}", payload);

    // Get the GitHub access token
    let token = get_github_token(&state).await.map_err(|err| {
        // Handle token retrieval error
        let error_message = format!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    // Create an instance of the GitHubClient
    let github_client = GitHubClient::new(
        state.config.files.repo_url.clone(),
        state.reqwest_client.clone(),
        token,
    );

    // Create the pull request using the new method from GitHubClient
    match github_client
        .create_pull_request(
            &payload.head_branch,
            &payload.base_branch,
            &payload.title,
            &payload.description,
        )
        .await
    {
        Ok(pull_request_url) => {
            // If the pull request creation is successful, respond with the pull request URL
            info!("Pull request created successfully from {} to {}", payload.head_branch, payload.base_branch);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Pull request created successfully".to_string(),
                    data: Some(CreatePRData { pull_request_url }),
                }),
            ))
        }
        Err(err) => {
            // Handle error case in creating the pull request
            let error_message = format!("Failed to create pull request: {:?}", err);
            error!("{}", error_message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}

/// Handler to check out or create a Git branch.
pub async fn checkout_or_create_branch_handler(
    State(state): State<AppState>,
    Path(branch_name): Path<String>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    info!("Checking out or creating branch: {}", branch_name);

    // Use the git interface to perform operations
    match state.git.checkout_or_create_branch(&branch_name) {
        Ok(_) => {
            info!("Successfully checked out/created branch: {}", branch_name);
            Ok((StatusCode::OK, format!("Successfully checked out/created branch: {}", branch_name)))
        },
        Err(err) => {
            error!("Error checking out/creating branch: {:?}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to checkout/create branch: {}", err)))
        }
    }
}

/// Handler to pull the latest changes for a specified branch.
pub async fn pull_handler(
    State(state): State<AppState>,
    Path(branch): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    info!("Received request to pull latest changes for branch '{}'", branch);

    // Attempt to pull the latest changes for the specified branch
    match state.git.git_pull_branch(&branch) {
        Ok(_) => {
            info!("Repository pulled successfully for branch '{}'.", branch);
            Ok((
                StatusCode::OK,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: format!("Repository pulled successfully for branch '{}'.", branch),
                    data: Some("Pull operation completed.".to_string()),
                }),
            ))
        },
        Err(err) => {
            error!("Failed to pull repository for branch '{}': {:?}", branch, err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to pull repository for branch '{}': {}", branch, err),
            ))
        }
    }
}

/// Handler for fetching the current branch of the repository.
pub async fn get_current_branch_handler(State(state): State<AppState>) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    info!("Received request to fetch current branch");

    // Use the git::Interface from AppState to get the current branch
    match state.git.get_current_branch().await {
        Ok(branch_name) => {
            info!("Current branch is: {}", branch_name);
            
            // Return the branch name in the response
            Ok((
                StatusCode::OK,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Current branch fetched successfully.".to_string(),
                    data: Some(branch_name),
                }),
            ))
        }
        Err(err) => {
            error!("Failed to get current branch: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get current branch: {}", err),
            ))
        }
    }
}

/// Route definitions for GitHub operations
pub async fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/branches", get(list_branches_handler))
        .route("/pulls", post(create_pull_request_handler))
        .route("/checkout/branches/:branch_name", put(checkout_or_create_branch_handler))
        .route("/pull/:branch", post(pull_handler))
        .route("/current-branch", get(get_current_branch_handler))
}
