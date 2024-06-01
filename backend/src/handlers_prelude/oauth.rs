use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use color_eyre::eyre::Context;
use log::{error, info};
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
    req: Request,
) -> Result<(HeaderMap, Redirect), (StatusCode, String)> {
    match get_oath_processor(&state, query, req).await {
        Ok(redirect) => Ok(redirect),
        Err(e) => {
            error!("An error was encountered during oauth processing: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "An error was encountered during oauth processing: {:?}",
                    e.to_string()
                ),
            ))
        }
    }
}

pub async fn get_oauth2_url(State(state): State<AppState>) -> String {
    // TODO: actually validate CSRF token
    // <https://discord.com/developers/docs/topics/oauth2#state-and-security>
    let (url, _token) = state.oauth.authorize_url(CsrfToken::new_random).url();
    url.to_string()
}

/// This is pretty stupid, but I want to be able to use `color_eyre::Result` and `?` for error handling, but
/// that doesn't directly implement axum's `IntoResponse`, but that just requires calling `.to_string()` on the error.
/// Async closures are unstable <https://github.com/rust-lang/rust/issues/62290> as of 2024-05-26
async fn get_oath_processor(
    state: &AppState,
    query: GetOAuthQuery,
    req: Request,
) -> color_eyre::Result<(HeaderMap, Redirect)> {
    // Support for dev and local environments, where discord sends the user
    // after the first step of the handshake
    let redirect_url = if cfg!(debug_assertions) {
        "http://localhost:8080/api/oauth".to_string()
    } else {
        let scheme = req.uri().scheme_str().map_or("http", |s| s);
        format!(
            "{}://{}/api/oauth",
            scheme,
            req.headers().get("host").unwrap().to_str()?
        )
    };

    // The obtained token after they authenticate
    let token_data: oauth2::StandardTokenResponse<_, _> = state
        .oauth
        .exchange_code(AuthorizationCode::new(query.code))
        .set_redirect_uri(std::borrow::Cow::Owned(RedirectUrl::new(redirect_url)?))
        .request_async(async_http_client)
        .await
        .wrap_err("OAuth token request failed")?;

    let token = token_data.access_token().secret();
    // Use that token to request user data
    let response = state
        .reqwest_client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(token_data.access_token().secret())
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

    let redirect = if cfg!(debug_assertions) {
        Redirect::to("http://localhost:5173/")
    } else {
        Redirect::to("/")
    };

    let mut headers = HeaderMap::new();
    // headers.append("Set-Cookie", format!("test=test; SameSite=None; Max-Age={}; Domain=localhost; Path=/", token_result.expires_in().unwrap().as_secs()).parse()?);
    // let the cookie be read from the vite build
    if cfg!(debug_assertions) {
        headers.append(
            "Set-Cookie",
            format!("access-token={token}; Secure; HttpOnly; Path=/;").parse()?,
        );
    } else {
        headers.append(
            "Set-Cookie",
            format!(
                "access-token={token}; Secure; HttpOnly; Path=/; Max-Age={}",
                token_data.expires_in().unwrap().as_secs()
            )
            .parse()?,
        );
    }
    Ok((headers, redirect))
}
