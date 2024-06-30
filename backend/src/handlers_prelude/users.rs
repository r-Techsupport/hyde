use axum::{extract::State, http::HeaderMap, Json};
use log::error;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{db::User, eyre_to_axum_err, perms::Permission, require_perms, AppState};

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionDetails {
    permission_tag: String,
    permission: Permission,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetUserResponse {
    id: i64,
    username: String,
    permissions: Vec<PermissionDetails>,
}

pub fn create_user_response(user: User, perms: &[Permission]) -> GetUserResponse {
    GetUserResponse {
        id: user.id,
        username: user.username,
        permissions: perms
            .iter()
            .map(|perm| PermissionDetails {
                permission_tag: String::from(*perm),
                permission: *perm,
            })
            .collect(),
    }
}

pub async fn get_users_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<GetUserResponse>>, (StatusCode, String)> {
    require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageUsers],
    )
    .await?;

    match state.db.get_all_users().await {
        Ok(users) => {
            let mut get_user_response = Vec::new();

            for user in users {
                let user_perms = &state
                    .db
                    .get_user_permissions(user.id)
                    .await
                    .map_err(eyre_to_axum_err)?;
                get_user_response.push(create_user_response(user, user_perms));
            }

            Ok(Json(get_user_response))
        }
        Err(e) => {
            error!("An error was encountered fetching all users: {e:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching all users, \
                    check server logs for more info"
                    .to_owned(),
            ))
        }
    }
}

pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<GetUserResponse>, (StatusCode, String)> {
    let user = require_perms(axum::extract::State(&state), headers, &[]).await?;

    let user_perms = &state
        .db
        .get_user_permissions(user.id)
        .await
        .map_err(eyre_to_axum_err)?;

    Ok(Json(create_user_response(user, user_perms)))
}
