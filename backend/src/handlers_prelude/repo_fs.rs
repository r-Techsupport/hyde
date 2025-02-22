//! Endpoints for interacting with the repository's filesystem (create doc/asset, read doc/asset, et cetera)
use crate::git::INode;
use axum::{
    body::Bytes,
    debug_handler,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::handlers_prelude::ApiError;
use crate::{perms::Permission, require_perms, AppState};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocQuery {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocResponse {
    pub contents: String,
}

async fn get_gh_token(state: &AppState) -> Result<String, ApiError> {
    match state.gh_client.get_token().await {
        Ok(token) => Ok(token),
        Err(e) => {
            error!("Failed to retrieve GitHub token: {e}");
            Err(ApiError::from((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())))
        }
    }
}

/// This handler accepts a `GET` request to `/api/doc?path=`.
/// TODO: refactor to pass it in directly as a url path instead of doing the whole url arguments thing
pub async fn get_doc_handler(
    State(state): State<AppState>,
    Query(query): Query<GetDocQuery>,
) -> Result<Json<GetDocResponse>, ApiError> {
    match state.git.get_doc(&query.path) {
        Ok(maybe_doc) => maybe_doc.map_or(
            Err(ApiError::from((
                StatusCode::NOT_FOUND,
                "The file at the provided path was not found.".to_string(),
            ))),
            |doc| Ok(Json(GetDocResponse { contents: doc })),
        ),
        Err(e) => {
            warn!(
                "Failed to fetch doc with path: {:?}; error: {:?}",
                query.path, e
            );
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Fetch failed, check server logs for more info".to_string(),
            )))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PutDocRequestBody {
    contents: String,
    path: String,
    commit_message: String,
    branch_name: String,
}

#[debug_handler]
pub async fn put_doc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PutDocRequestBody>,
) -> Result<StatusCode, ApiError> {
    let author = require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageContent],
    )
    .await?;

    // Generate commit message combining author and default update message
    let default_commit_message = format!("{} updated {}", author.username, body.path);
    let final_commit_message = format!("{}\n\n{}", default_commit_message, body.commit_message);

    // Use the branch name from the request body
    let branch_name = &body.branch_name;

    match state.git.put_doc(
        &body.path,
        &body.contents,
        &final_commit_message,
        &get_gh_token(&state).await?,
        branch_name,
    ) {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            error!("Failed to complete put_doc call with error: {e:?}");
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create document, check server logs for more info".to_string(),
            )))
        }
    }
}

/// Deletes the document at the provided path, if the user has perms.
pub async fn delete_doc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GetDocQuery>,
) -> Result<StatusCode, ApiError> {
    let author = require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageContent],
    )
    .await?;

    state
        .git
        .delete_doc(
            &query.path,
            &format!("{} deleted {}", author.username, query.path),
            &get_gh_token(&state).await?,
        )
        .map_err(|e| {
            error!("Failed to delete doc: {:?}", e);
            ApiError::from((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// This handler reads the document folder and builds a tree style object
/// representing the state of the tree. This is used in the viewer for directory navigation.
pub async fn get_doc_tree_handler(
    State(state): State<AppState>,
) -> Result<Json<INode>, ApiError> {
    match state.git.get_doc_tree() {
        Ok(t) => Ok(Json(t)),
        Err(e) => {
            error!("An error was encountered fetching the document tree: {e:?}");
            Err(ApiError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching the doc tree, check server logs for more info",
            )))
        }
    }
}

/// This handler reads the assets folder and builds a tree style object
/// representing the state of the tree. This is used in the viewer for directory navigation.
pub async fn get_asset_tree_handler(
    State(state): State<AppState>,
) -> Result<Json<INode>, (StatusCode, &'static str)> {
    match state.git.get_asset_tree() {
        Ok(t) => Ok(Json(t)),
        Err(e) => {
            error!("An error was encountered fetching the asset tree: {e:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching the asset tree, \
                    check server logs for more info",
            ))
        }
    }
}

/// This handler fetches an asset from the repo's asset folder
pub async fn get_asset_handler(
    State(state): State<AppState>,
    Path(path): Path<Vec<String>>,
) -> Result<impl IntoResponse, ApiError> {
    let file_name = path.last().unwrap().clone();
    let path = path.join("/");

    // Attempt to retrieve the asset, returning an ApiError on failure
    let file = match state.git.get_asset(&path)? {
        Some(file) => file,
        None => {
            return Err(ApiError::from((
                StatusCode::NOT_FOUND,
                format!("File not found: {}", path),
            )));
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        format!("image/{}", file_name.split_once(".").unwrap().1)
            .parse()
            .unwrap(),
    );
    headers.insert(
        CONTENT_DISPOSITION,
        format!("inline; filename={file_name:?}").parse().unwrap(),
    );

    Ok((headers, file))
}


/// This handler creates or replaces the asset at the provided path
/// with a new asset
pub async fn put_asset_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<Vec<String>>,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let path = path.join("/");
    let author = require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageContent],
    )
    .await?;
    // Generate commit message combining author and default update message
    let message = format!("{} updated {}", author.username, path);

    // Call put_asset to update the asset, passing the required parameters
    state
        .git
        .put_asset(&path, &body, &message, &get_gh_token(&state).await?)?;

    Ok(StatusCode::CREATED)
}

/// This handler creates or replaces the asset at the provided path
/// with a new asset
pub async fn delete_asset_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<Vec<String>>,
) -> Result<StatusCode, ApiError> {
    let path = path.join("/");
    let author = require_perms(State(&state), headers, &[Permission::ManageContent]).await?;
    // Generate commit message combining author and default update message
    let message = format!("{} deleted {}", author.username, path);
    state
        .git
        .delete_asset(&path, &message, &get_gh_token(&state).await?)?;

    Ok(StatusCode::OK)
}

pub async fn create_tree_route() -> Router<AppState> {
    Router::new()
        .route("/tree/doc", get(get_doc_tree_handler))
        .route(
            "/doc",
            get(get_doc_handler)
                .put(put_doc_handler)
                .delete(delete_doc_handler),
        )
        .route("/tree/asset", get(get_asset_tree_handler))
        .route(
            "/asset/{*path}",
            get(get_asset_handler)
                .put(put_asset_handler)
                .delete(delete_asset_handler),
        )
        // 256 MiB
        .layer(DefaultBodyLimit::max(
            (256_u32 * (2_u32.pow(20))).try_into().unwrap(),
        ))
}
