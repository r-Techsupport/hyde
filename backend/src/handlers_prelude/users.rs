use axum::routing::{delete, get, post};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::handlers_prelude::ApiError;
use crate::{
    db::{Database, Group, User},
    perms::Permission,
    require_perms, AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    avatar_url: String,
    groups: Vec<Group>,
    permissions: Vec<Permission>,
}

pub async fn create_user_response(
    db: &Database,
    user: User,
) -> Result<UserResponse, ApiError> {
    let groups = db
        .get_user_groups(user.id)
        .await?;

    let permissions = db
        .get_user_permissions(user.id)
        .await?;

    Ok(UserResponse {
        id: user.id,
        username: user.username,
        avatar_url: user.avatar_url,
        groups,
        permissions,
    })
}

pub async fn get_users_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    match state.db.get_all_users().await {
        Ok(users) => {
            let mut get_users_response = Vec::new();

            for user in users {
                get_users_response.push(create_user_response(&state.db, user).await?);
            }

            Ok(Json(get_users_response))
        }
        Err(e) => {
            error!("An error was encountered fetching all users: {e:?}");
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching all users, \
                    check server logs for more info"
                    .to_owned(),
            )))
        }
    }
}

pub async fn get_current_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, ApiError> {
    let user = require_perms(axum::extract::State(&state), headers, &[]).await?;
    Ok(Json(create_user_response(&state.db, user).await?))
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserGroupsRequestBody {
    group_ids: Vec<i64>,
}

pub async fn post_user_membership_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(body): Json<UpdateUserGroupsRequestBody>,
) -> Result<Json<UserResponse>, ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    for group_id in body.group_ids {
        state
            .db
            .add_group_membership(group_id, user_id)
            .await?;
    }

    let user = state
        .db
        .get_user(user_id)
        .await?
        .unwrap();

    Ok(Json(create_user_response(&state.db, user).await?))
}

pub async fn delete_user_membership_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(body): Json<UpdateUserGroupsRequestBody>,
) -> Result<Json<UserResponse>, ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    for group_id in body.group_ids {
        state
            .db
            .remove_group_membership(group_id, user_id)
            .await?;
    }

    let user = state
        .db
        .get_user(user_id)
        .await?
        .unwrap();

    Ok(Json(create_user_response(&state.db, user).await?))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> Result<(), ApiError> {
    require_perms(State(&state), headers, &[Permission::ManageUsers]).await?;

    match state.db.delete_user(user_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to delete user with ID {}: {}", user_id, e);
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred while deleting the user, check server logs for more info",
            )))
        }
    }
}

pub async fn delete_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let user = require_perms(axum::extract::State(&state), headers, &[]).await?;

    match state.db.delete_user(user.id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to delete current user with ID {}: {}", user.id, e);
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred while deleting the user, check server logs for more info",
            )))
        }
    }
}

pub async fn create_user_route() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_users_handler))
        .route(
            "/users/groups/{user_id}",
            post(post_user_membership_handler).delete(delete_user_membership_handler),
        )
        .route("/users/{user_id}", delete(delete_user_handler))
        .route(
            "/users/me",
            get(get_current_user_handler).delete(delete_current_user),
        )
}
