use axum::response::Redirect;
use color_eyre::eyre::Context;
use color_eyre::Result;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use std::env;
use url::Url;

#[derive(Clone)]
pub struct OAathClient {
    client: BasicClient,
}

impl OAathClient {
    pub fn new() -> Result<Self> {
        let client_id = env::var("OAUTH_CLIENT_ID").context("OAUTH_CLIENT_ID not set in env")?;
        let client_secret = env::var("OAUTH_SECRET").context("OAUTH_SECRET not sent in env")?;
        let auth_url = env::var("OAUTH_URL").context("OAUTH_URL not set in env")?;
        let token_url = env::var("OAUTH_TOKEN_URL").context("OAUTH_TOKEN_URL not set in env")?;

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url)?,
            Some(TokenUrl::new(token_url)?),
        );
        // .set_redirect_uri(RedirectUrl::new("http://redirect".to_string())?);
        Ok(Self { client })
    }

    /// This function returns a URL, and a CsrfToken to be used when a request to the
    /// oauth handler is received. This should be called when first initiating a login request.
    pub fn get_auth_url(&self) -> (Url, CsrfToken) {
        self.client.authorize_url(CsrfToken::new_random).url()
        // println!("Browse to: {}", auth_url);
        // let token_result = self.client
        // .exchange_code(AuthorizationCode::new("some authorization code".to_string()))
        // // Set the PKCE code verifier.
        // .request_async(async_http_client)
        // .await.unwrap();
        // println!("Result of auth: {:?}", token_result);
        // todo!();
    }

    pub async fn oauth_token_handler(&self, code: String, csrf_token: String) {
        // Now you can trade it for an access token.
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_redirect_uri(std::borrow::Cow::Owned(
                RedirectUrl::new("http://127.0.0.1:8080/api/oauth".to_string()).unwrap(),
            ))
            .request_async(async_http_client)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        println!("{token_result:#?}");
        println!(
            "{:#?}",
            client
                .get("https://discord.com/api/v10/users/@me")
                .bearer_auth(token_result.access_token().secret())
                .header("User-Agent", "DiscordBot (https://github.com/r-Techsupport/rts-crm, 1.0)")
                .send()
                .await
        );
        // println!("extra fields: {:?}", token_result.extra_fields());
        // return Redirect::permanent("http://127.0.0.1:5173");
    }
}

// // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
// // token URL.
// let client =
//     BasicClient::new(
//         ClientId::new("client_id".to_string()),
//         Some(ClientSecret::new("client_secret".to_string())),
//         AuthUrl::new("http://authorize".to_string())?,
//         Some(TokenUrl::new("http://token".to_string())?)
//     )
//     // Set the URL the user will be redirected to after the authorization process.
//     .set_redirect_uri(RedirectUrl::new("http://redirect".to_string())?);

// // Generate a PKCE challenge.
// let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

// // Generate the full authorization URL.
// let (auth_url, csrf_token) = client
//     .authorize_url(CsrfToken::new_random)
//     // Set the desired scopes.
//     .add_scope(Scope::new("read".to_string()))
//     .add_scope(Scope::new("write".to_string()))
//     // Set the PKCE code challenge.
//     .set_pkce_challenge(pkce_challenge)
//     .url();

// // This is the URL you should redirect the user to, in order to trigger the authorization
// // process.
// println!("Browse to: {}", auth_url);

// // Once the user has been redirected to the redirect URL, you'll have access to the
// // authorization code. For security reasons, your code should verify that the `state`
// // parameter returned by the server matches `csrf_state`.

// // Now you can trade it for an access token.
// let token_result = client
//     .exchange_code(AuthorizationCode::new("some authorization code".to_string()))
//     // Set the PKCE code verifier.
//     .set_pkce_verifier(pkce_verifier)
//     .request_async(async_http_client)
//     .await?;

// // Unwrapping token_result will either produce a Token or a RequestTokenError.
