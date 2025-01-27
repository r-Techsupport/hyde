use axum::routing::get;
use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    Router,
};
use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat};
use oauth2::{AuthorizationCode, CsrfToken, RedirectUrl, TokenResponse};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{db::User, AppState};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOAuthQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordUserObject {
    username: String,
    id: String,
    avatar: Option<String>,
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
            error!("An error was encountered during oauth processing: {:#?}", e);
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
    // after the first step of the handshake.
    // HTTPS is required in production for full functionality so it's ok to hardcode that in
    let redirect_url = if cfg!(debug_assertions) {
        format!(
            "http://{}/api/oauth",
            req.headers().get("host").unwrap().to_str()?
        )
    } else {
        format!(
            "https://{}/api/oauth",
            req.headers().get("host").unwrap().to_str()?
        )
    };
    // The obtained token after they authenticate
    let token_data: oauth2::StandardTokenResponse<_, _> = state
        .oauth
        .exchange_code(AuthorizationCode::new(query.code))
        .set_redirect_uri(std::borrow::Cow::Owned(RedirectUrl::new(redirect_url)?))
        .request_async(&state.reqwest_client)
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
            "DiscordBot (https://github.com/r-Techsupport/hyde, 0)",
        )
        .send()
        .await?;
    // https://discord.com/developers/docs/resources/user#user-object
    let discord_user_info: DiscordUserObject = serde_json::from_slice(&response.bytes().await?)?;
    let avatar_url = if let Some(hash) = discord_user_info.avatar {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{hash}.png",
            discord_user_info.id
        )
    } else {
        "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    };
    // https://discord.com/developers/docs/reference#image-formatting
    let all_users = state.db.get_all_users().await?;
    let expiration_date = Utc::now()
        + token_data
            .expires_in()
            .wrap_err("Discord OAuth2 response didn't include an expiration date")?;
    // Update the user entry if one is already there, otherwise create a user
    if let Some(existing_user) = all_users
        .iter()
        .find(|u| u.username == discord_user_info.username)
    {
        state
            .db
            .update_user(&User {
                id: existing_user.id,
                username: existing_user.username.clone(),
                token: token.to_string(),
                expiration_date: expiration_date.to_rfc3339(),
                avatar_url,
            })
            .await?;
        info!("User {:?} re-authenticated", existing_user.username);
    } else {
        state
            .db
            .create_user(
                discord_user_info.username.to_string(),
                token.to_string(),
                expiration_date.to_rfc3339(),
                avatar_url,
            )
            .await?;
        info!(
            "New user {:?} authenticated, entry added to database",
            discord_user_info.username
        );
    }
    // If the user is the admin specified in the config, give them the admin role
    let admin_username = &state.config.discord.admin_username;
    let all_users = state.db.get_all_users().await?;
    let maybe_admin_user = all_users.iter().find(|u| u.username == *admin_username);
    if let Some(admin_user) = maybe_admin_user {
        let their_groups = state.db.get_user_groups(admin_user.id).await?;
        // If they don't have the admin group, add it
        if !their_groups.iter().any(|g| g.name == "Admin") {
            let all_groups = state.db.get_all_groups().await?;
            let admin_group = all_groups
                .into_iter()
                .find(|g| g.name == "Admin")
                .expect("No admin group in database");
            state
                .db
                .add_group_membership(admin_group.id, admin_user.id)
                .await?;
            info!("User {admin_username:?} was automatically added to the admin group based off of the server config");
        }
    }

    // After authenticating, send them back to the homepage
    let redirect = if cfg!(debug_assertions) {
        Redirect::to("http://localhost:5173/")
    } else {
        Redirect::to("/")
    };

    let mut headers = HeaderMap::new();
    headers.append(
        "Set-Cookie",
        format!(
            "access-token={token}; Secure; HttpOnly; Path=/; Max-Age={}",
            token_data.expires_in().unwrap().as_secs()
        )
        .parse()?,
    );
    headers.append(
        "Set-Cookie",
        format!(
            "username={}; Path=/; Max-Age={}",
            discord_user_info.username,
            token_data.expires_in().unwrap().as_secs()
        )
        .parse()?,
    );
    Ok((headers, redirect))
}

pub async fn create_oauth_route() -> Router<AppState> {
    Router::new()
        .route("/oauth", get(get_oauth2_handler))
        .route("/oauth/url", get(get_oauth2_url))
}
