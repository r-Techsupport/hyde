use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use log::info;
use oauth2::{reqwest::async_http_client, AuthorizationCode, RedirectUrl};
use oauth2::{CsrfToken, TokenResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOAuthQuery {
    pub code: String,
    pub state: String,
}

/// This endpoint is used for authentication, and it's required to implement oauth2. Users
/// are sent here by discord after they authenticate, then they're redirected to the homepage
pub async fn get_oauth2_handler(
    State(state): State<AppState>,
    Query(query): Query<GetOAuthQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    match get_oath_processor(&state, query).await {
        Ok(redirect) => Ok(redirect),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_oauth2_url(State(state): State<AppState>) -> String {
    // TODO: actually validate CSRF token
    // <https://discord.com/developers/docs/topics/oauth2#state-and-security>
    let (url, _token) = state.oauth.authorize_url(CsrfToken::new_random).url();
    url.to_string()
}

/// This is pretty stupid, but I want to be able to use color_eyre::Result and `?` for error handling, but
/// that doesn't directly implement axum's IntoResponse, but that just requires calling .to_string() on the error.
/// Async closures are unstable <https://github.com/rust-lang/rust/issues/62290> as of 2024-05-26
async fn get_oath_processor(
    state: &AppState,
    query: GetOAuthQuery,
) -> color_eyre::Result<Redirect> {
    // Support for dev and local environments, where discord sends the user
    // after the first step of the handshake
    let redirect_url = if cfg!(debug_assertions) {
        "http://127.0.0.1:8080/api/oauth".to_string()
    } else {
        "/api/oauth".to_string()
    };

    // The obtained token after they authenticate
    let token_result = state
        .oauth
        .exchange_code(AuthorizationCode::new(query.code))
        .set_redirect_uri(std::borrow::Cow::Owned(RedirectUrl::new(redirect_url)?))
        .request_async(async_http_client)
        .await?;

    // Use that token to request user data
    let response = state
        .reqwest_client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(token_result.access_token().secret())
        .header(
            "User-Agent",
            "DiscordBot (https://github.com/r-Techsupport/rts-crm, 0)",
        )
        .send()
        .await?;
    if response.status().is_success() {
        // While there's no actual infrastructure to make use of this information,
        // we got it (hooray)
        let user_json: Value = serde_json::from_slice(&response.bytes().await?)?;
        info!(
            "New user authenticated (WIP, not complete), user: {:?}",
            user_json
        );
    };

    // redirect the user to localhost:5173 if they're running from a dev environment
    #[cfg(debug_assertions)]
    return color_eyre::Result::Ok(Redirect::temporary("http://localhost:5173/"));
    // This is reachable, but the above macro evaluates to return when the linter is linting
    #[allow(unreachable_code)]
    color_eyre::Result::Ok(Redirect::temporary("/"))
}
