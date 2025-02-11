use crate::handlers_prelude::ApiError;
use crate::AppState;
use axum::routing::{get, post, put};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

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

/// Fetches the list of branches from a GitHub repository.
pub async fn list_branches_handler(
    State(state): State<AppState>,
) -> Result<Json<BranchesData>, ApiError> {
    // Fetch the branch details from GitHub using the GitHubClient instance
    let branch_details = state.gh_client.list_branches().await?;

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

    // Wrap the branches data in the BranchesData struct and return as JSON
    info!("Branches fetched successfully.");
    Ok(Json(BranchesData { branches }))
}

/// Handler to create a pull request from a specified head branch to a base branch.
pub async fn create_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreatePRRequest>,
) -> Result<(StatusCode, Json<CreatePRData>), ApiError> {
    let pull_request_url = state
        .gh_client
        .create_pull_request(
            &payload.head_branch,
            &payload.base_branch,
            &payload.title,
            &payload.description,
            payload.issue_numbers,
        )
        .await?;

    info!(
        "Pull request created successfully from {} to {}",
        payload.head_branch, payload.base_branch
    );

    Ok((StatusCode::CREATED, Json(CreatePRData { pull_request_url })))
}

/// Handler to update a pull request from a specified head branch to a base branch.
pub async fn update_pull_request_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePRRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    // Update the pull request
    let updated_pr_url = state
        .gh_client
        .update_pull_request(
            payload.pr_number,
            payload.title.as_deref(),
            payload.description.as_deref(),
            payload.base_branch.as_deref(),
            payload.issue_numbers,
        )
        .await?;

        info!("Pull request #{} updated successfully", payload.pr_number);
        Ok((StatusCode::OK, Json(updated_pr_url)))
        
}

/// Handler to close a pull request.
pub async fn close_pull_request_handler(
    State(state): State<AppState>,
    Path(pr_number): Path<u64>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    // Attempt to close the pull request
    state
        .gh_client
        .close_pull_request(pr_number)
        .await?;

    info!("Pull request #{} closed successfully", pr_number);
    Ok((
        StatusCode::OK,
        Json(format!("Pull request #{} closed.", pr_number)),
    ))
}

/// Handler to check out or create a Git branch.
pub async fn checkout_or_create_branch_handler(
    State(state): State<AppState>,
    Path(branch_name): Path<String>,
) -> Result<(StatusCode, String), ApiError> {
    state
        .git
        .checkout_or_create_branch(&branch_name)?;

    info!("Successfully checked out/created branch: {}", branch_name);
    Ok((
        StatusCode::OK,
        format!("Successfully checked out/created branch: {}", branch_name),
    ))
}

/// Handler to pull the latest changes for a specified branch.
pub async fn pull_handler(
    State(state): State<AppState>,
    Path(branch): Path<String>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    state
        .git
        .git_pull_branch(&branch)?;

    info!("Repository pulled successfully for branch '{}'.", branch);
    Ok((
        StatusCode::OK,
        Json(format!(
            "Repository pulled successfully for branch '{}'.",
            branch
        )),
    ))
}

/// Handler for fetching the current branch of the repository.
pub async fn get_current_branch_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    // Use the git::Interface from AppState to get the current branch
    let branch_name = state
        .git
        .get_current_branch()
        .await?;

    info!("Current branch is: {}", branch_name);
    Ok((StatusCode::OK, Json(branch_name)))
}

/// Handler for fetching the default branch of the repository.
pub async fn get_default_branch_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let default_branch = state
        .gh_client
        .get_default_branch()
        .await?;

    info!("Default branch is: {}", default_branch);
    Ok((StatusCode::OK, Json(default_branch)))
}

/// Handler to fetch issues from a GitHub repository.
pub async fn get_issues_handler(
    State(state): State<AppState>,
    Path(state_param): Path<String>,
) -> Result<(StatusCode, Json<IssuesData>), ApiError> {
    // Fetch issues using the GitHub client
    let issues = state
        .gh_client
        .get_issues(Some(state_param.as_str()), None)
        .await?;

    info!("Issues fetched successfully.");
    let response = IssuesData { issues };

    Ok((StatusCode::OK, Json(response)))
}

/// Route definitions for GitHub operations
pub async fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/branches", get(list_branches_handler))
        .route("/pulls", post(create_pull_request_handler))
        .route(
            "/checkout/branches/{branch_name}",
            put(checkout_or_create_branch_handler),
        )
        .route("/pulls/update", put(update_pull_request_handler))
        .route(
            "/pull-requests/{pr_number}/close",
            post(close_pull_request_handler),
        )
        .route("/pull/{branch}", post(pull_handler))
        .route("/current-branch", get(get_current_branch_handler))
        .route("/issues/{state}", get(get_issues_handler))
        .route("/repos/default-branch", get(get_default_branch_handler))
}
