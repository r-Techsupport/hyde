//! Handler for GitHub last commit.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use crate::{AppState, git::Interface};
use serde_json::json;

async fn get_last_commit_handler(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let repo = state.git.repo();

    // Lock the Mutex to access the repo
    let repo_locked = repo.lock().unwrap();

    // Call the find_last_commit function while the lock is held
    let commit_result = Interface::find_last_commit(&*repo_locked);

    match commit_result {
        Ok(commit) => {
            let response = json!({
                "latest_commit": commit.message().unwrap_or("No message").to_string(),
            });
            tracing::info!("Last commit found: {:?}", response);
            Ok((StatusCode::OK, Json(response))) // Return Status and JSON response
        }
        Err(e) => {
            // Log the error for debugging purposes
            tracing::error!("Failed to retrieve the last commit: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR) // Return only StatusCode on error
        }
    }
}

/// Creates the route for getting the last commit
pub async fn create_last_commit_route() -> Router<AppState> {
    Router::new()
        .route("/last_commit", get(get_last_commit_handler).options(options_handler))
}

// Options handler for CORS preflight
async fn options_handler() -> impl IntoResponse {
    tracing::info!("OPTIONS request received");
    (StatusCode::NO_CONTENT, ())
}
