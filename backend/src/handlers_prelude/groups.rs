use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    db::{Database, Group},
    eyre_to_axum_err,
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
) -> Result<GroupResponse, (StatusCode, String)> {
    let permissions = db
        .get_group_permissions(group.id)
        .await
        .map_err(eyre_to_axum_err)?;

    let members = db
        .get_group_members(group.id)
        .await
        .map_err(eyre_to_axum_err)?;

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
            .collect::<Vec<_>>(),
    })
}

pub async fn get_groups_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<GroupResponse>>, (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    match state.db.get_all_groups().await {
        Ok(groups) => {
            let mut get_groups_response = Vec::new();

            for group in groups {
                get_groups_response.push(create_group_response(&state.db, group).await?);
            }

            Ok(Json(get_groups_response))
        }
        Err(e) => {
            error!("An error was encountered fetching all groups: {e:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching all groups, \
                    check server logs for more info"
                    .to_owned(),
            ))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGroupRequestBody {
    group_name: String,
}

pub async fn post_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateGroupRequestBody>,
) -> Result<Json<GroupResponse>, (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    Ok(Json(
        create_group_response(
            &state.db,
            state
                .db
                .create_group(body.group_name)
                .await
                .map_err(eyre_to_axum_err)?,
        )
        .await?,
    ))
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
) -> Result<Json<GroupResponse>, (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    let current_permissions = state
        .db
        .get_group_permissions(group_id)
        .await
        .map_err(eyre_to_axum_err)?;

    let new_permissions = body.permissions;

    let permissions_to_remove = current_permissions
        .iter()
        .filter(|perm| !new_permissions.contains(perm))
        .collect::<Vec<_>>();

    let permissions_to_add = new_permissions
        .iter()
        .filter(|perm| !current_permissions.contains(perm))
        .collect::<Vec<_>>();

    for perm in permissions_to_remove {
        state
            .db
            .remove_group_permission(group_id, *perm)
            .await
            .map_err(eyre_to_axum_err)?;
    }

    for perm in permissions_to_add {
        state
            .db
            .add_group_permission(group_id, *perm)
            .await
            .map_err(eyre_to_axum_err)?;
    }

    Ok(Json(
        create_group_response(
            &state.db,
            state
                .db
                .get_group(group_id)
                .await
                .map_err(eyre_to_axum_err)?
                .unwrap(),
        )
        .await?,
    ))
}

pub async fn delete_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<i64>,
) -> Result<(), (StatusCode, String)> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    state
        .db
        .delete_group(group_id)
        .await
        .map_err(eyre_to_axum_err)
}
