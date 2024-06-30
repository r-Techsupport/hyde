use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use log::error;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    db::{Database, Group, User},
    eyre_to_axum_err,
    perms::Permission,
    require_perms, AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionDetails {
    permission_tag: String,
    permission: Permission,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetUserResponse {
    id: i64,
    username: String,
    groups: Vec<Group>,
    permissions: Vec<PermissionDetails>,
}

pub async fn create_user_response(
    db: &Database,
    user: User,
) -> Result<GetUserResponse, (StatusCode, String)> {
    let groups = db
        .get_user_groups(user.id)
        .await
        .map_err(eyre_to_axum_err)?;

    let perms = db
        .get_user_permissions(user.id)
        .await
        .map_err(eyre_to_axum_err)?;

    Ok(GetUserResponse {
        id: user.id,
        username: user.username,
        groups,
        permissions: perms
            .iter()
            .map(|perm| PermissionDetails {
                permission_tag: String::from(*perm),
                permission: *perm,
            })
            .collect(),
    })
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
                get_user_response.push(create_user_response(&state.db, user).await?);
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

pub async fn get_current_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<GetUserResponse>, (StatusCode, String)> {
    let user = require_perms(axum::extract::State(&state), headers, &[]).await?;
    Ok(Json(create_user_response(&state.db, user).await?))
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserGroupsRequestBody {
    group_ids: Vec<i64>,
}

pub async fn put_user_membership_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(body): Json<UpdateUserGroupsRequestBody>,
) -> Result<Json<GetUserResponse>, (StatusCode, String)> {
    require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageUsers],
    )
    .await?;

    for group_id in body.group_ids {
        state
            .db
            .add_group_membership(group_id, user_id)
            .await
            .map_err(eyre_to_axum_err)?;
    }

    let user = state
        .db
        .get_user(user_id)
        .await
        .map_err(eyre_to_axum_err)?
        .unwrap();

    Ok(Json(create_user_response(&state.db, user).await?))
}

pub async fn delete_user_membership_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(body): Json<UpdateUserGroupsRequestBody>,
) -> Result<Json<GetUserResponse>, (StatusCode, String)> {
    require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageUsers],
    )
    .await?;

    for group_id in body.group_ids {
        state
            .db
            .remove_group_membership(group_id, user_id)
            .await
            .map_err(eyre_to_axum_err)?;
    }

    let user = state
        .db
        .get_user(user_id)
        .await
        .map_err(eyre_to_axum_err)?
        .unwrap();

    Ok(Json(create_user_response(&state.db, user).await?))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> Result<(), (StatusCode, String)> {
    require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageUsers],
    )
    .await?;

    state
        .db
        .delete_user(user_id)
        .await
        .map_err(eyre_to_axum_err)
}

pub async fn delete_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(), (StatusCode, String)> {
    let user = require_perms(axum::extract::State(&state), headers, &[]).await?;

    state
        .db
        .delete_user(user.id)
        .await
        .map_err(eyre_to_axum_err)
}
