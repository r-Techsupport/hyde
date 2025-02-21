//! Github Webhook events are sent here

use axum::routing::post;
use axum::{extract::State, http::HeaderMap, Router};
use tracing::{debug, info};

use crate::handlers_prelude::ApiError;
use crate::AppState;

pub async fn github_hook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let event_type = headers.get("x-github-event").unwrap().to_str().unwrap();

    debug!("Received Github webhook event of type {:?}", event_type);

    if event_type == "push" {
        info!("New changes pushed to Github, pulling changes...");
        state.git.pull()?;
    }

    Ok(())
}

pub async fn create_github_route() -> Router<AppState> {
    Router::new().route("/hooks/github", post(github_hook_handler))
}
