use axum::routing::post;
use axum::{extract::State, http::HeaderMap, Router};

use crate::{perms::Permission, AppState};

use super::{ApiError, require_perms};

pub async fn post_reclone_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;
    state.git.reclone()?;
    Ok(())
}

pub async fn create_reclone_route() -> Router<AppState> {
    Router::new().route("/reclone", post(post_reclone_handler))
}
