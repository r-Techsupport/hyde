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

pub async fn list_branches_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // Get the access token
    let token = state.gh_credentials.get(&state.reqwest_client).await.map_err(|err| {
        error!("Failed to get GitHub token: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get access token"})))
    })?;

    // Call the function to fetch branches (repo_name is derived inside the function)
    let branches = list_branches(&state.reqwest_client, &token).await.map_err(|err| {
        error!("Error fetching branches: {:?}", err);
        let error_message = format!("Failed to fetch branches: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": error_message})))
    })?;

    info!("Fetched branches successfully.");
    Ok((StatusCode::OK, Json(json!(branches))))
}

// Update the route to handle GET requests for branches
pub async fn list_github_branches_route() -> Router<AppState> {
    Router::new().route("/branches", get(list_branches_handler))
}
