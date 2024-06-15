use std::collections::HashMap;

use axum::{
    debug_handler,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, ContextCompat};
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};

use crate::{db::User, eyre_to_axum_err, perms::Permission, AppState};

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
}

#[debug_handler]
pub async fn put_doc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PutDocRequestBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let maybe_user: Option<FoundUser> =
        find_user(&state, headers).await.map_err(eyre_to_axum_err)?;
    let author: User;
    // Unauthorized means that the client has not identified itself, where as forbidden
    // means that the client has identified itself, and it does not have the required permissions
    match maybe_user {
        Some(user) => match user {
            FoundUser::ExpiredUser(u) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    format!(
                    "The access token has expired for the user {}, they must authenticate again.",
                    u.username
                ),
                ))
            }
            FoundUser::User(u) => {
                let groups = &state
                    .db
                    .get_user_groups(u.id)
                    .await
                    .map_err(eyre_to_axum_err)?;
                let mut has_permission = false;
                for group in groups {
                    if state
                        .db
                        .group_has_permission(group.id, Permission::ManageContent)
                        .await
                        .map_err(eyre_to_axum_err)?
                    {
                        has_permission = true;
                    }
                }
                if has_permission {
                    author = u;
                } else {
                    return Err((
                        StatusCode::FORBIDDEN,
                        format!(
                            "User {:?} lacks the permission to edit documents.",
                            u.username
                        ),
                    ));
                }
            }
        },
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "No valid user is authenticated.".to_string(),
            ))
        }
    };

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
    match state.git.put_doc(
        &body.path,
        &body.contents,
        &format!("{} updated {}", author.username, body.path),
        &gh_token,
    ) {
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

// This stuff should probably be moved but I don't know where.
/// The output of a find_user call, used to differentiate between expired users and valid users
enum FoundUser {
    ExpiredUser(User),
    User(User),
}

/// Find the user attached to a particular request, if there is one, and the access token is still valid
async fn find_user(state: &AppState, headers: HeaderMap) -> color_eyre::Result<Option<FoundUser>> {
    let mut cookies: HashMap<&str, &str> = HashMap::new();
    // There can be multiple cookie headers, and each cookie header can contain multiple cookies
    let cookie_headers = headers.get_all("Cookie");
    for cookie_header in cookie_headers {
        let deserialized_cookie = cookie_header
            .to_str()
            .wrap_err("Cookie header contains invalid UTF-8")?;
        for nv_pair in deserialized_cookie.split("; ") {
            let (name, value) = nv_pair.split_once('=').wrap_err("Malformed cookie")?;
            cookies.insert(name, value);
        }
    }
    if let Some(token) = cookies.get("access-token") {
        trace!("Request was made that contains an access-token cookie");
        if let Some(user) = state.db.get_user_from_token(token.to_string()).await? {
            let expiration_date = DateTime::parse_from_rfc3339(&user.expiration_date)
                .wrap_err("Expiration time in database is not a valid time")?;
            if expiration_date < Utc::now() {
                debug!("User {:?} made a request that requires a valid access token but their access token expired", user.username);
                return Ok(Some(FoundUser::ExpiredUser(user)));
            } else {
                debug!("User {:?} made a request that requires a valid access token and they have a valid access token", user.username);
                return Ok(Some(FoundUser::User(user)));
            }
        } else {
            trace!("No user was found in the database with the request's access token");
        }
    } else {
        trace!("Request was made that lacked an access-token cookie");
    }

    Ok(None)
}
