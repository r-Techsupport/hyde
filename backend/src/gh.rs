//! Code for interacting with GitHub (authentication, prs, et cetera)

use chrono::DateTime;
use color_eyre::eyre::Context;
use color_eyre::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// In order to authenticate as a github app or generate an installation access token, you must generate a JSON Web Token (JWT). The JWT must contain predefined *claims*.
///
/// <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app#about-json-web-tokens-jwts>
/// <https://docs.rs/jsonwebtoken/latest/jsonwebtoken/fn.encode.html>
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// *Issued At*; The time that the JWT was created.
    ///
    /// To protect against clock drift, we recommend that you set this
    /// 60 seconds in the past and ensure that your server's date and time
    /// is set accurately (for example, by using the Network Time Protocol).
    ///
    /// Stored as the number of seconds since the epoch
    iat: u64,
    /// *Expires At*; The expiration time of the JWT, after which it can't
    /// be used to request an installation token. Must be less than or equal to 10 minutes.
    ///
    /// Stored as the number of seconds since the epoch.
    exp: u64,
    /// *Issuer*; The client ID or application ID of your GitHub App.
    ///
    /// This value is used to find the right public key to verify the signature of the JWT.
    /// You can find your app's IDs on the settings page for your GitHub App.
    /// Use of the client ID is recommended.
    iss: String,
    /// *Message authentication code algorithm*; This should be RS256 since your JWT must be signed using the RS256 algorithm.
    alg: String,
}

impl Claims {
    pub fn new() -> Result<Self> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let iat = current_time - 60;
        let exp = current_time + (60 * 5);
        let iss = env::var("GH_CLIENT_ID").wrap_err("Failed to read the `GH_CLIENT_ID` env var")?;

        Ok(Self {
            iat,
            exp,
            iss,
            alg: "RS256".to_string(),
        })
    }
}

/// A wrapper around the github access token that automatically refreshes if the token has been invalidated
#[derive(Clone)]
pub struct GithubAccessToken {
    expires_at: SystemTime,
    token: String,
}

impl GithubAccessToken {
    /// Initialize, but don't fetch a token yet.
    pub fn new() -> Self {
        Self {
            expires_at: UNIX_EPOCH,
            token: String::new(),
        }
    }

    /// Return the cached token if it's less than one hour old, or fetch a new token from the api, and return that, updating the cache
    pub async fn get(&mut self, req_client: &Client) -> Result<String> {
        // Fetch a new token if more than 59 minutes have passed
        // Tokens expire after 1 hour, this is to account for clock drift
        if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() > (60 * 59) {
            let api_response = get_access_token(req_client).await?;
            self.token = api_response.0;
            self.expires_at = api_response.1;
        }
        Ok(self.token.clone())
    }
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    expires_at: String,
    token: String,
}

/// Request a github installation access token using the provided reqwest client.
/// The installation access token will expire after 1 hour.
/// Returns the new token, and the time of expiration
async fn get_access_token(req_client: &Client) -> Result<(String, SystemTime)> {
    let token = gen_jwt_token()?;
    let response = req_client
        .post(format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            get_installation_id(req_client).await?
        ))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "rts-crm")
        // https://docs.github.com/en/rest/about-the-rest-api/api-versions?apiVersion=2022-11-28
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    let deserialized_response: AccessTokenResponse =
        serde_json::from_slice(&response.bytes().await?)?;
    Ok((
        deserialized_response.token,
        DateTime::parse_from_rfc3339(&deserialized_response.expires_at)?.into(),
    ))
}

#[derive(Deserialize)]
struct InstallationIdResponse {
    id: u64,
}

/// Fetch the Installation ID. This value is required for most API calls
///
/// <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation#generating-an-installation-access-token>
async fn get_installation_id(req_client: &Client) -> Result<String> {
    let response = req_client
        .get("https://api.github.com/app/installations")
        .bearer_auth(gen_jwt_token()?)
        .header("User-Agent", "rts-crm")
        // https://docs.github.com/en/rest/about-the-rest-api/api-versions?apiVersion=2022-11-28
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    let deserialized_response =
        &serde_json::from_slice::<Vec<InstallationIdResponse>>(&response.bytes().await?)?[0];
    return Ok(deserialized_response.id.to_string());
}

/// Generate a new JWT token for use with github api interactions.
fn gen_jwt_token() -> Result<String> {
    let mut private_key_file = File::open("cms-data/key.pem")
        .wrap_err("Failed to read private key from `cms-data/key.pem`")?;
    let mut private_key = Vec::new();
    private_key_file.read_to_end(&mut private_key)?;
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &Claims::new()?,
        &EncodingKey::from_rsa_pem(&private_key)?,
    )?)
}
