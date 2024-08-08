//! Github Webhook events are sent here

use axum::{extract::State, http::HeaderMap};
use tracing::{error, info};

use crate::AppState;


pub async fn github_hook_handler(State(state): State<AppState>, headers: HeaderMap) {
    let event_type = headers.get("x-github-event").unwrap().to_str().unwrap();
        if event_type == "push" {
            info!("New changes pushed to Github, pulling changes...");
            match state.git.pull() {
                Ok(_) => {
                },
                Err(e) => {
                    error!("Failed to auto-pull changes with error: {e:?}");
                },
            }

        }
}