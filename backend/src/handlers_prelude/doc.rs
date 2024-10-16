use axum::{debug_handler, extract::{Query, State}, http::{HeaderMap, StatusCode}, Json, Router};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{perms::Permission, require_perms, AppState};

use super::eyre_to_axum_err;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocQuery {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocResponse {
    pub contents: String,
}

/// This handler accepts a `GET` request to `/api/doc?path=`.
pub async fn get_doc_handler(
    State(state): State<AppState>,
    Query(query): Query<GetDocQuery>,
) -> Result<Json<GetDocResponse>, (StatusCode, &'static str)> {
    match state.git.get_doc(&query.path) {
        Ok(maybe_doc) => maybe_doc.map_or(
            Err((
                StatusCode::NOT_FOUND,
                "The file at the provided path was not found.",
            )),
            |doc| Ok(Json(GetDocResponse { contents: doc })),
        ),
        Err(e) => {
            warn!(
                "Failed to fetch doc with path: {:?}; error: {:?}",
                query.path, e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Fetch failed, check server logs for more info",
            ))
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

pub async fn put_doc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PutDocRequestBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Unauthorized means that the client has not identified itself, where as forbidden
    // means that the client has identified itself, and it does not have the required permissions
    let author = require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageContent],
    )
    .await?;

    let gh_token = match &state.gh_credentials.get(&state.reqwest_client).await {
        Ok(t) => t.clone(),
        Err(e) => {
            error!("Failed to authenticate with github for a put_doc request with error: {e:?}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to authenticate with github to push changes".to_string(),
            ));
        }
    };

    // Generate commit message combining author and default update message
    let default_commit_message = format!("{} updated {}", author.username, body.path);
    let final_commit_message = format!("{}\n\n{}", default_commit_message, body.commit_message);

    // Use the branch name from the request body
    let branch_name = &body.branch_name;

    match state
        .git
        .put_doc(&body.path, &body.contents, &final_commit_message, &gh_token, branch_name)
    {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            error!("Failed to complete put_doc call with error: {e:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create document, check server logs for more info".to_string(),
            ))
        }
    }
}

pub async fn delete_doc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GetDocQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let author = require_perms(
        axum::extract::State(&state),
        headers,
        &[Permission::ManageContent],
    )
    .await?;

    let gh_token = state.gh_credentials.get(&state.reqwest_client).await.unwrap();
    state.git.delete_doc(&query.path, &format!("{} deleted {}", author.username, query.path), &gh_token).map_err(eyre_to_axum_err)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_doc_route() -> Router<AppState> {
    Router::new().route("/doc", get(get_doc_handler)
        .put(put_doc_handler)
        .delete(delete_doc_handler))
}
