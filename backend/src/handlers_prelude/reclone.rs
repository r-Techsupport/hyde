use axum::{extract::State, http::HeaderMap, Router};
use axum::routing::post;
use reqwest::StatusCode;

use crate::{perms::Permission, AppState};

use super::{eyre_to_axum_err, require_perms};

pub async fn post_reclone_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(), (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;
    state.git.reclone(&state.config.files.repo_url, &state.config.files.repo_path).map_err(eyre_to_axum_err)?;
    Ok(())
}

pub async fn create_reclone_route() -> Router<AppState> {
    Router::new()
        .route("/reclone", post(post_reclone_handler))
}
