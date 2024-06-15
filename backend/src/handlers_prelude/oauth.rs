use std::env;

use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat};
use log::{debug, error, info, warn};
use oauth2::{reqwest::async_http_client, AuthorizationCode, RedirectUrl};
use oauth2::{CsrfToken, TokenResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    db,
    AppState,
};

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
        let discord_user_json: Value = serde_json::from_slice(&response.bytes().await?)?;
        let username = discord_user_json
            .get("username")
            .wrap_err("Discord API response did not contain a `username` field")?
            .as_str()
            .wrap_err(
                "The `username` field from the discord api response did not contain a string",
            )?;
        let all_users = db::get_all_users(&state.db_connection_pool).await?;
        // If the user doesn't already exist, create one
        if !all_users.iter().any(|u| u.username == username) {
            let expiration_date = Utc::now()
                + token_data
                    .expires_in()
                    .wrap_err("Discord OAuth2 response didn't include an expiration date")?;
            db::create_user(
                &state.db_connection_pool,
                username.to_string(),
                token.to_string(),
                expiration_date.to_rfc3339(),
            )
            .await?;
            info!("New user {username:?} authenticated, entry added to database");
        }
        // If the user is the admin specified in the config, give them the admin role
        if let Ok(admin_username) = env::var("ADMIN_USERNAME") {
            let all_users = db::get_all_users(&state.db_connection_pool).await?;
            let maybe_admin_user = all_users.iter().find(|u| u.username == admin_username);
            if let Some(admin_user) = maybe_admin_user {
                let their_groups =
                    db::get_user_groups(&state.db_connection_pool, admin_user.id).await?;
                // If they don't have the admin group, add it
                if !their_groups.iter().any(|g| g.name == "Admin") {
                    let admin_group = db::get_all_groups(&state.db_connection_pool)
                        .await?
                        .into_iter()
                        .find(|g| g.name == "Admin")
                        .expect("No admin group in database");
                    db::add_group_membership(
                        &state.db_connection_pool,
                        admin_group.id,
                        admin_user.id,
                    )
                    .await?;
                    debug!("User {admin_username:?} was automatically added to the admin group based off of the server config");
                }
                // if let Some(admin_group) = their_groups.iter().find(|g| g.name == "Admin") {
                //     if add_group_membership(&state.db_connection_pool, admin_group.id, admin_user.id).await? {
                //         debug!("{admin_username:?} was added automatically to the Admin group based off of the server config.");
                //     }
                // }
            }
        } else {
            warn!("The \"ADMIN_USERNAME\" environment variable is not set, no default admin will be available.");
        }
    }
    // After authenticating, send them back to the homepage
    let redirect = if cfg!(debug_assertions) {
        Redirect::to("http://localhost:5173/")
    } else {
        Redirect::to("/")
    };

    let mut headers = HeaderMap::new();
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
