use axum::routing::{delete, get, put};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    db::{Database, Group},
    handlers_prelude::ApiError,
    perms::Permission,
    require_perms, AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Member {
    id: i64,
    username: String,
    avatar_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupResponse {
    id: i64,
    name: String,
    permissions: Vec<Permission>,
    members: Vec<Member>,
}

pub async fn create_group_response(
    db: &Database,
    group: Group,
) -> Result<GroupResponse, ApiError> {
    let permissions = db.get_group_permissions(group.id).await?;
    let members = db.get_group_members(group.id).await?;

    Ok(GroupResponse {
        id: group.id,
        name: group.name,
        permissions,
        members: members
            .into_iter()
            .map(|m| Member {
                id: m.id,
                username: m.username,
                avatar_url: m.avatar_url,
            })
            .collect(),
    })
}

pub async fn get_groups_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<GroupResponse>>, ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    let groups = state.db.get_all_groups().await?;
    let mut get_groups_response = Vec::with_capacity(groups.len());

    for group in groups {
        get_groups_response.push(create_group_response(&state.db, group).await?);
    }

    Ok(Json(get_groups_response))
}

#[derive(Serialize, Deserialize)]
pub struct CreateGroupRequestBody {
    group_name: String,
}

pub async fn post_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateGroupRequestBody>,
) -> Result<Json<GroupResponse>, ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    let group = state.db.create_group(body.group_name).await?;
    let response = create_group_response(&state.db, group).await?;

    Ok(Json(response))
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGroupPermissionsRequestBody {
    permissions: Vec<Permission>,
}

pub async fn put_group_permissions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<i64>,
    Json(body): Json<UpdateGroupPermissionsRequestBody>,
) -> Result<Json<GroupResponse>, ApiError> {
    // Ensure the user has the necessary permissions
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    // Fetch current permissions for the group
    let current_permissions = state.db.get_group_permissions(group_id).await?;
    let new_permissions = body.permissions;

    // Identify permissions to remove and add
    let permissions_to_remove = current_permissions
        .iter()
        .filter(|perm| !new_permissions.contains(perm))
        .collect::<Vec<_>>();

    let permissions_to_add = new_permissions
        .iter()
        .filter(|perm| !current_permissions.contains(perm))
        .collect::<Vec<_>>();

    // Remove permissions
    for perm in permissions_to_remove {
        state.db.remove_group_permission(group_id, *perm).await?;
    }

    // Add permissions
    for perm in permissions_to_add {
        state.db.add_group_permission(group_id, *perm).await?;
    }

    // Fetch updated group and return the response
    let updated_group = state.db.get_group(group_id).await?.ok_or_else(|| {
        ApiError::from((StatusCode::NOT_FOUND, "Group not found in the database".to_string()))
    })?;

    Ok(Json(create_group_response(&state.db, updated_group).await?))
}

pub async fn delete_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<i64>,
) -> Result<(), ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    state.db.delete_group(group_id).await?;

    Ok(())
}

pub async fn create_group_route() -> Router<AppState> {
    Router::new()
        .route("/groups", get(get_groups_handler).post(post_group_handler))
        .route("/groups/{group_id}", delete(delete_group_handler))
        .route(
            "/groups/{group_id}/permissions",
            put(put_group_permissions_handler),
        )
}
