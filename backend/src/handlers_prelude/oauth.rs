use axum::routing::get;
use axum::{
    Router,
    extract::{Query, Request, State},
    http::HeaderMap,
    response::Redirect,
};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use color_eyre::eyre::Context;
use jiff::Timestamp;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

use crate::{AppState, db::User};

use super::ApiError;

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

#[derive(Debug, Deserialize, Serialize)]
struct ExchangeCodeData {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: String,
    redirect_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

/// This endpoint is used for authentication, and it's required to implement oauth2. Users
/// are sent here by discord after they authenticate, then they're redirected to the homepage
pub async fn get_oauth2_handler(
    State(state): State<AppState>,
    Query(query): Query<GetOAuthQuery>,
    req: Request,
) -> Result<(HeaderMap, Redirect), ApiError> {
    // Support for dev and local environments, where discord sends the user
    // after the first step of the handshake.
    // HTTPS is required in production for full functionality so it's ok to hardcode that in
    let redirect_url = if cfg!(debug_assertions) {
        format!(
            "http://{}/api/oauth",
            req.headers().get("host").unwrap().to_str().unwrap()
        )
    } else {
        format!(
            "https://{}/api/oauth",
            req.headers().get("host").unwrap().to_str().unwrap()
        )
    };

    // "In accordance with the relevant RFCs, the token and token revocation URLs will only
    // accept a content type of `application/x-www-form-urlencoded`. JSON content is not
    // permitted and will return an error."
    let mut headers = HeaderMap::new();
    headers.append(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let data = ExchangeCodeData {
        client_id: state.config.oauth.discord.client_id.clone(),
        client_secret: state.config.oauth.discord.secret.clone(),
        grant_type: "authorization_code".to_string(),
        code: query.code,
        redirect_uri: redirect_url,
    };

    // The obtained token after they authenticate
    let token_data: TokenResponse = state
        .reqwest_client
        .post("https://discord.com/api/v10/oauth2/token")
        .headers(headers)
        .form(&data)
        .send()
        .await
        .wrap_err("OAuth token request failed")?
        .json()
        .await
        .wrap_err("Failed to parse token response")?;

    let auth_token = token_data.access_token;
    let discord_user_info = fetch_discord_user(&state, &auth_token).await?;
    // https://discord.com/developers/docs/reference#image-formatting
    let avatar_url = if let Some(hash) = discord_user_info.avatar {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{hash}.png",
            discord_user_info.id
        )
    } else {
        "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    };
    let all_users = state.db.get_all_users().await?;
    let expiration_date = Timestamp::now() + Duration::from_secs(token_data.expires_in);

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
                token: auth_token.to_string(),
                expiration_date: expiration_date.to_string(),
                avatar_url,
            })
            .await?;
        info!("User {:?} re-authenticated", existing_user.username);
    } else {
        state
            .db
            .create_user(
                discord_user_info.username.to_string(),
                auth_token.to_string(),
                expiration_date.to_string(),
                avatar_url,
            )
            .await?;
        info!(
            "New user {:?} authenticated, entry added to database",
            discord_user_info.username
        );
    }

    // Now's a good time to check if they're an admin
    authorize_config_admin(state).await?;

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
            "access-token={auth_token}; Secure; HttpOnly; Path=/; Max-Age={}",
            token_data.expires_in
        )
        .parse()
        .wrap_err("Failed to create access token cookie")?,
    );
    headers.append(
        "Set-Cookie",
        format!(
            "username={}; Path=/; Max-Age={}",
            discord_user_info.username, token_data.expires_in
        )
        .parse()
        .wrap_err("Failed to create auth cookie")?,
    );
    Ok((headers, redirect))
}

pub async fn get_oauth2_url(State(state): State<AppState>) -> String {
    // TODO: actually validate CSRF token
    // <https://discord.com/developers/docs/topics/oauth2#state-and-security>
    let mut csrf_token = String::with_capacity(16);
    let bytes: [u8; 16] = rand::rng().random();
    BASE64_URL_SAFE_NO_PAD.encode_string(bytes, &mut csrf_token);
    info!("csrf token: {}", csrf_token);
    format!(
        "{}&state={}",
        state.config.oauth.discord.url.clone(),
        csrf_token
    )
}

/// Fetches a list of information about a user using a valid oauth access token
async fn fetch_discord_user(
    state: &AppState,
    access_token: &str,
) -> color_eyre::Result<DiscordUserObject> {
    // Use that token to request user data
    let response = state
        .reqwest_client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(access_token)
        .header(
            "User-Agent",
            "DiscordBot (https://github.com/r-Techsupport/hyde, 0)",
        )
        .send()
        .await
        .wrap_err("Failed to fetch user data from Discord API")?;
    // https://discord.com/developers/docs/resources/user#user-object
    let discord_user_info: DiscordUserObject = serde_json::from_slice(
        &response
            .bytes()
            .await
            .expect("Discord API responds completely"),
    )
    .wrap_err("Discord API returned an invalid user info object")?;

    Ok(discord_user_info)
}

/// Ensures that the user defined as Admin within Hyde's config has the Admin role
async fn authorize_config_admin(state: AppState) -> color_eyre::Result<()> {
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
            info!(
                "User {admin_username:?} was automatically added to the admin group based off of the server config"
            );
        }
    }
    Ok(())
}

pub async fn create_oauth_route() -> Router<AppState> {
    Router::new()
        .route("/oauth", get(get_oauth2_handler))
        .route("/oauth/url", get(get_oauth2_url))
}
