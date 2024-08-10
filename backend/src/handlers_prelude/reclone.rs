use axum::{extract::State, http::HeaderMap};
use reqwest::StatusCode;

use crate::{perms::Permission, AppState};

use super::{eyre_to_axum_err, require_perms};

pub async fn post_reclone_handler(State(state): State<AppState>, headers: HeaderMap) -> Result<(), (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;
    state.git.reclone().map_err(eyre_to_axum_err)?;
    Ok(())
}
