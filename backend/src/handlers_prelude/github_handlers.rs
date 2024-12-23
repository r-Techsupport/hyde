use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json, Router,
};
use axum::routing::{get, post, put};
use tracing::{error, info};
use serde::{Serialize, Deserialize};
use serde_json::Value;
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
    pub issue_numbers: Option<Vec<u64>>,
}

#[derive(Serialize, Debug)]
pub struct IssuesData {
    pub issues: Vec<Value>,
}

#[derive(Serialize)]
pub struct Issue {
    pub id: u64,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePRRequest {
    pub pr_number: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_branch: Option<String>,
    pub issue_numbers: Option<Vec<u64>>,
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
            payload.issue_numbers,
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

pub async fn update_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePRRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    info!("Received request to update pull request: {:?}", payload);

    // Get the GitHub access token
    let token = get_github_token(&state).await.map_err(|err| {
        let error_message = format!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    // Create an instance of the GitHubClient
    let github_client = GitHubClient::new(
        state.config.files.repo_url.clone(),
        state.reqwest_client.clone(),
        token,
    );

    // Update the pull request
    match github_client
        .update_pull_request(
            payload.pr_number,
            payload.title.as_deref(),
            payload.description.as_deref(),
            payload.base_branch.as_deref(),
            payload.issue_numbers,
        )
        .await
    {
        Ok(updated_pr_url) => {
            info!("Pull request #{} updated successfully", payload.pr_number);
            Ok((
                StatusCode::OK,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Pull request updated successfully.".to_string(),
                    data: Some(updated_pr_url),
                }),
            ))
        }
        Err(err) => {
            let error_message = format!("Failed to update pull request: {:?}", err);
            error!("{}", error_message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}

pub async fn close_pull_request_handler(
    State(state): State<AppState>,
    Path(pr_number): Path<u64>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    info!("Received request to close pull request #{}", pr_number);

    // Get the GitHub access token
    let token = get_github_token(&state).await.map_err(|err| {
        let error_message = format!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    // Create an instance of the GitHubClient
    let github_client = GitHubClient::new(
        state.config.files.repo_url.clone(),
        state.reqwest_client.clone(),
        token,
    );

    // Attempt to close the pull request
    match github_client.close_pull_request(pr_number).await {
        Ok(_) => {
            info!("Pull request #{} closed successfully", pr_number);
            Ok((
                StatusCode::OK,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Pull request closed successfully.".to_string(),
                    data: Some(format!("Pull request #{} closed.", pr_number)),
                }),
            ))
        }
        Err(err) => {
            let error_message = format!("Failed to close pull request: {:?}", err);
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

/// Handler for fetching the default branch of the repository.
pub async fn get_default_branch_handler(State(state): State<AppState>) -> Result<(StatusCode, Json<ApiResponse<String>>), (StatusCode, String)> {
    info!("Received request to fetch default branch");

    // Get the GitHub access token
    let token = get_github_token(&state).await.map_err(|err| {
        let error_message = format!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    })?;

    // Create an instance of the GitHubClient
    let github_client = GitHubClient::new(
        state.config.files.repo_url.clone(),
        state.reqwest_client.clone(),
        token,
    );
    

    // Use the `get_default_branch` method from the `Gh` struct in AppState
    match github_client.get_default_branch().await {
        Ok(default_branch) => {
            info!("Default branch is: {}", default_branch);
            
            // Return the default branch name in the response
            Ok((
                StatusCode::OK,
                Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Default branch fetched successfully.".to_string(),
                    data: Some(default_branch),
                }),
            ))
        }
        Err(err) => {
            error!("Failed to get default branch: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get default branch: {}", err),
            ))
        }
    }
}

/// Handler to fetch issues from a GitHub repository.
pub async fn get_issues_handler(
    State(state): State<AppState>,
    Path(state_param): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<IssuesData>>), (StatusCode, String)> {
    info!("Received request to fetch issues");

    let state_param = state_param.as_str();

    // Get the GitHubClient instance
    let github_client = GitHubClient::new(
        state.config.files.repo_url.clone(),
        state.reqwest_client.clone(),
        get_github_token(&state).await.map_err(|err| {
            let error_message = format!("Failed to get GitHub token: {:?}", err);
            error!("{}", error_message);  // Log the error here
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?,
    );

    // Fetch issues using the GitHub client
    match github_client
        .get_issues(Some(state_param), None)
        .await
    {
        Ok(issues) => {
            info!("Issues fetched successfully.");
            let response = ApiResponse {
                status: "success".to_string(),
                message: "Issues fetched successfully.".to_string(),
                data: Some(IssuesData { issues }),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(err) => {
            // Log and return an error
            let error_message = format!("Failed to fetch issues: {:?}", err);
            error!("{}", error_message);  // Log the error here
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}

/// Route definitions for GitHub operations
pub async fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/branches", get(list_branches_handler))
        .route("/pulls", post(create_pull_request_handler))
        .route("/checkout/branches/:branch_name", put(checkout_or_create_branch_handler))
        .route("/pulls/update", put(update_pull_request_handler))
        .route("/pull-requests/:pr_number/close", post(close_pull_request_handler))
        .route("/pull/:branch", post(pull_handler))
        .route("/current-branch", get(get_current_branch_handler))
        .route("/issues/:state", get(get_issues_handler))
        .route("/repos/default-branch", get(get_default_branch_handler))
}
