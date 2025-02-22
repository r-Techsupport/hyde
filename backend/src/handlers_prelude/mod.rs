//! All Axum handlers are exported from this module

use std::collections::HashMap;

use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::HeaderMap};
use chrono::{DateTime, Utc};
mod repo_fs;
pub use repo_fs::*;
mod oauth;
pub use oauth::*;
mod users;
pub use users::*;
mod groups;
pub use groups::*;
mod logout;
pub use logout::*;
mod github_hook;
pub use github_hook::*;
mod reclone;
pub use reclone::*;
mod github_handlers;
pub use github_handlers::*;

use color_eyre::{
    eyre::{self, Context, ContextCompat},
    Report,
};
use reqwest::StatusCode;
use tracing::{debug, trace};

use crate::{db::User, perms::Permission, AppState};

pub struct ApiError {
    status: Option<StatusCode>,
    error: Report,
}

impl ApiError {
    pub fn new(status: Option<StatusCode>, error: impl Into<Report>) -> Self {
        Self {
            status,
            error: error.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, self.error.to_string()).into_response()
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        Self {
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
            error: eyre::eyre!(message),
        }
    }
}

impl From<(StatusCode, String)> for ApiError {
    fn from((status, message): (StatusCode, String)) -> Self {
        Self {
            status: Some(status),
            error: eyre::eyre!(message),
        }
    }
}

impl From<&str> for ApiError {
    fn from(message: &str) -> Self {
        Self {
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
            error: eyre::eyre!(message.to_string()),
        }
    }
}

impl From<(StatusCode, &str)> for ApiError {
    fn from((status, message): (StatusCode, &str)) -> Self {
        Self {
            status: Some(status),
            error: eyre::eyre!(message.to_string()),
        }
    }
}

impl From<Report> for ApiError {
    fn from(error: Report) -> Self {
        Self {
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
            error,
        }
    }
}

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

/// This function is used to add permissions to endpoints.
///
/// When placed at the top of an Axum handler, you can specify permission(s)
/// to require. If they are missing from the user, it will return an error,
/// which you can propagate through the handler with `?`.
// TODO: Write unit tests for this. May require refactoring so that
// it only needs a database, instead of the whole app state
pub async fn require_perms(
    State(state): State<&AppState>,
    headers: HeaderMap,
    perms: &[Permission],
) -> Result<User, ApiError> {
    let maybe_user = find_user(state, headers).await?;
    match maybe_user {
        Some(user) => match user {
            FoundUser::ExpiredUser(u) => Err(ApiError::new(
                Some(StatusCode::UNAUTHORIZED),
                eyre::eyre!(
                    "The access token has expired for the user {}, they must authenticate again.",
                    u.username
                ),
            )),
            FoundUser::User(u) => {
                let user_perms = &state
                    .db
                    .get_user_permissions(u.id)
                    .await?;
                let has_permissions = perms.iter().all(|perm| user_perms.contains(perm));
                if has_permissions {
                    Ok(u)
                } else {
                    Err(ApiError::new(
                        Some(StatusCode::FORBIDDEN),
                        eyre::eyre!(
                            "User {:?} lacks the permission to edit documents.",
                            u.username
                        ),
                    ))
                }
            }
        },
        None => Err(ApiError::new(
            Some(StatusCode::UNAUTHORIZED),
            eyre::eyre!("No valid user is authenticated, perhaps you forgot to add `{{credentials: \"include\"}}` in your fetch options?"),
        )),
    }
}
