//! Github Webhook events are sent here

use axum::routing::post;
use axum::{Router, extract::State, http::HeaderMap};
use tracing::{debug, error, info};

use crate::AppState;

pub async fn github_hook_handler(State(state): State<AppState>, headers: HeaderMap) {
    let event_type = headers.get("x-github-event").unwrap().to_str().unwrap();
    debug!("Received Github webhook event of type {event_type:?}");
    if event_type == "push" {
        info!("New changes pushed to Github, pulling changes...");
        match state.git.pull() {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to auto-pull changes with error: {e:?}");
            }
        }
    }
}

pub async fn create_github_route() -> Router<AppState> {
    Router::new().route("/hooks/github", post(github_hook_handler))
}
