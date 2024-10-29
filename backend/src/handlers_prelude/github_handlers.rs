use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json, Router,
};
use axum::routing::{get, post, put};
use tracing::{error,info};
use serde::{Serialize, Deserialize};
use crate::gh::{get_all_branch_details, create_pull_request};
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
///
/// This asynchronous function accesses the application state to fetch
/// the GitHub access token using the provided HTTP client. If the token
/// retrieval fails, it returns an error with a corresponding HTTP status code
/// and a descriptive error message.
///
/// # Parameters
/// - `state`: A reference to the `AppState`, which contains the necessary
///   credentials and the HTTP client for making the request.
///
/// # Returns
/// - On success, returns a `Result` containing the GitHub access token as a `String`.
/// - On failure, returns a tuple containing the appropriate `StatusCode` and an error message.
///
/// # Errors
/// This function can fail if there are issues with the credentials store
/// or if the token cannot be retrieved for any reason.
async fn get_github_token(state: &AppState) -> Result<String, (StatusCode, String)> {
    state.gh_credentials.get(&state.reqwest_client, &state.config.oauth.github.client_id).await.map_err(|err| {
        eyre_to_axum_err(err)
    })
}

/// Fetches the list of branches from a GitHub repository.
///
/// This function interacts with the GitHub API to retrieve all branches
/// for a specific repository. It requires an access token for authentication.
///
/// # Parameters
/// - `reqwest_client`: An instance of `reqwest::Client` used to make HTTP requests.
/// - `token`: A string slice representing the GitHub access token used for authentication.
///
/// # Returns
/// - On success, returns a `Result` containing a vector of branch names as strings.
/// - On failure, returns an error that implements the `std::error::Error` trait, indicating
///   what went wrong during the API request.
///
/// # Errors
/// This function can fail due to network issues, invalid tokens, or other API-related errors.
pub async fn list_branches_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<BranchesData>>), (StatusCode, String)> {
    info!("Received request to fetch branches");

    // Get the GitHub access token
    let token = get_github_token(&state).await?;

    // Fetch the branch details from GitHub
    let branch_details = get_all_branch_details(&state.reqwest_client, &token).await.map_err(|err| {
        eyre_to_axum_err(err)
    })?; // Use string error

    // Extract branch names and protection status
    let branches: Vec<String> = branch_details
        .into_iter()
        .map(|branch| {
            let name = branch.name; // Adjust as necessary based on your structure
            // You can also check for protection status if needed
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
///
/// # Arguments
/// - `state` - Application state containing configuration and necessary clients (e.g., GitHub credentials and HTTP client).
/// - `payload` - A `CreatePRRequest` containing the following fields:
///   - `head_branch` - The name of the branch containing the changes to be merged.
///   - `base_branch` - The name of the branch into which the changes should be merged.
///   - `title` - The title of the pull request.
///   - `description` - A description of the changes in the pull request.
///
/// # Returns
/// On success, returns `StatusCode::CREATED` and a JSON message indicating success.
/// On failure, returns an appropriate `StatusCode` and a JSON error message.
///
/// # Errors
/// - Returns `StatusCode::INTERNAL_SERVER_ERROR` if there is an issue obtaining the GitHub token or creating the pull request.
/// - Logs errors if encountered during the process and returns a corresponding error message in the response.
pub async fn create_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreatePRRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreatePRData>>), (StatusCode, String)> {
    info!("Received create pull request request: {:?}", payload);

    // Get the GitHub access token
    let token = get_github_token(&state).await?;

    // Create the pull request on GitHub
    let pull_request_url = create_pull_request(
        &state.reqwest_client,
        &token,
        &payload.head_branch,
        &payload.base_branch,
        &payload.title,
        &payload.description,
    ).await.map_err(|err| {
        eyre_to_axum_err(err)
    })?;

    // Return success response
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

pub async fn checkout_or_create_branch_handler(
    State(state): State<AppState>,
    Path(branch_name): Path<String>, // Use Path extractor for the branch name
) -> Result<(StatusCode, String), (StatusCode, String)> {
    info!("Checking out or creating branch: {}", branch_name);

    let repo = state.git.get_repo();

    // Call your checkout_or_create_branch function through the Interface
    match state.git.checkout_or_create_branch(&repo, &branch_name) {
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

/// Route definitions for GitHub operations
pub async fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/branches", get(list_branches_handler))
        .route("/pulls", post(create_pull_request_handler))
        .route("/check-out/branches/:branch_name", put(checkout_or_create_branch_handler))
        .route("/pull/:branch", post(pull_handler))
}