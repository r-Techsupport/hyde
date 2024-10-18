//! Getting Github Branches
use axum::{
    extract::State,
    http::StatusCode,
    Json, Router,
};
use axum::routing::get;
use tracing::{error, info};
use serde_json::json;
use crate::gh::list_branches;
use crate::AppState;
use color_eyre::Result;

/// Fetches the list of branches from a GitHub repository.
///
/// This function interacts with the GitHub API to retrieve all branches
/// for a specific repository. It requires an access token for authentication.
///
/// # Parameters
///
/// - `reqwest_client`: An instance of `reqwest::Client` used to make HTTP requests.
/// - `token`: A string slice representing the GitHub access token used for authentication.
///
/// # Returns
///
/// - On success, returns a `Result` containing a vector of branch names as strings.
/// - On failure, returns an error that implements the `std::error::Error` trait, indicating
///   what went wrong during the API request.
///
/// # Errors
///
/// This function can fail due to network issues, invalid tokens, or other API-related errors.
/// It is the caller's responsibility to handle these errors appropriately.

pub async fn list_branches_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // Get the access token
    let token_result = state.gh_credentials.get(&state.reqwest_client).await;

    let token = match token_result {
        Ok(token) => token,
        Err(err) => {
            error!("Failed to get GitHub token: {:?}", err);
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get access token"})),
            ));
        }
    };

    // Call the function to fetch branches (repo_name is derived inside the function)
    let branches_result = list_branches(&state.reqwest_client, &token).await;

    let branches = match branches_result {
        Ok(branches) => branches,
        Err(err) => {
            error!("Error fetching branches: {:?}", err);
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch branches: {:?}", err)})),
            ));
        }
    };

    info!("Fetched branches successfully.");
    Ok((StatusCode::OK, Json(json!(branches))))
}

// Update the route to handle GET requests for branches
pub async fn list_github_branches_route() -> Router<AppState> {
    Router::new().route("/branches", get(list_branches_handler))
}
