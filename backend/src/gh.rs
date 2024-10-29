//! Code for interacting with GitHub (authentication, prs, et cetera)

use chrono::DateTime;
use color_eyre::eyre::{bail, Context};
use color_eyre::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use dotenvy::dotenv;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{ info, error };

const GITHUB_API_URL: &str = "https://api.github.com";

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
    pub fn new(client_id: &str) -> Result<Self> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let iat = current_time - 60;
        let exp = current_time + (60 * 5);
        let iss = client_id.to_string();

        Ok(Self {
            iat,
            exp,
            iss,
            alg: "RS256".to_string(),
        })
    }
}

/// A wrapper around the github access token that automatically refreshes if the token has been invalidated
#[derive(Clone, Debug)]
pub struct GithubAccessToken {
    expires_at: Arc<Mutex<SystemTime>>,
    token: Arc<Mutex<String>>,
}

impl GithubAccessToken {
    /// Initialize, but don't fetch a token yet.
    pub fn new() -> Self {
        Self {
            // I don't know a better way to handle interior mutability
            expires_at: Arc::new(Mutex::new(UNIX_EPOCH)),
            token: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Return the cached token if it's less than one hour old, or fetch a new token from the api, and return that, updating the cache
    pub async fn get(&self, req_client: &Client, client_id: &str) -> Result<String> {
        let mut token_ref = self.token.lock().await;
        // Fetch a new token if more than 59 minutes have passed
        // Tokens expire after 1 hour, this is to account for clock drift
        if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() > (60 * 59) {
            let api_response = get_access_token(req_client, client_id).await?;
            *token_ref = api_response.0;
            let mut expires_ref = self.expires_at.lock().await;
            *expires_ref = api_response.1;
        }
        Ok(token_ref.clone())
    }
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    expires_at: String,
    token: String,
}

#[derive(Deserialize)]
pub struct Branch {
    pub name: String,
    pub protected: bool,
}

/// Request a github installation access token using the provided reqwest client.
/// The installation access token will expire after 1 hour.
/// Returns the new token, and the time of expiration
async fn get_access_token(req_client: &Client, client_id: &str) -> Result<(String, SystemTime)> {
    let token = gen_jwt_token(client_id)?;
    let response = req_client
        .post(format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            get_installation_id(req_client, client_id).await?
        ))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Hyde")
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
async fn get_installation_id(req_client: &Client, client_id: &str) -> Result<String> {
    let response = req_client
        .get("https://api.github.com/app/installations")
        .bearer_auth(gen_jwt_token(client_id)?)
        .header("User-Agent", "Hyde")
        // https://docs.github.com/en/rest/about-the-rest-api/api-versions?apiVersion=2022-11-28
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;
    // Validate that there's only one repo the app is installed on
    let repo_list =
        &serde_json::from_slice::<Vec<InstallationIdResponse>>(&response.bytes().await?)?;
    if repo_list.len() != 1 {
        bail!(
            "Hyde must only be installed on one repo, Github currently reports {} repos",
            repo_list.len()
        );
    }
    Ok(repo_list[0].id.to_string())
}

/// Generate a new JWT token for use with github api interactions.
fn gen_jwt_token(client_id: &str) -> Result<String> {
    let mut private_key_file = File::open("hyde-data/key.pem")
        .wrap_err("Failed to read private key from `hyde-data/key.pem`")?;
    let mut private_key = Vec::new();
    private_key_file.read_to_end(&mut private_key)?;
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &Claims::new(client_id)?,
        &EncodingKey::from_rsa_pem(&private_key)?,
    )?)
}

/// Retrieve the repository name from the `REPO_URL` environment variable.
/// 
/// The `REPO_URL` must be in the format `<owner>/<repo>.git`. This function removes
/// the `.git` suffix and parses the owner and repo name from the URL.
/// 
/// # Returns
/// A `Result` containing the repository name in the format `<owner>/<repo>`, or an error if
/// the environment variable is not set or the format is invalid.
/// 
/// # Errors
/// This function will return an error if `REPO_URL` is not set or has an invalid format.
fn get_repo_name_from_env() -> Result<String> {
    // Load environment variables from the .env file
    dotenv().ok();

    // Retrieve the repository URL from the environment variable
    let repo_url = env::var("REPO_URL")
        .context("REPO_URL must be set in the .env file")?;

    // Parse the repository name from the URL
    let repo_path = repo_url
        .trim_end_matches(".git")  // Remove the .git suffix
        .rsplit('/')  // Split by '/'
        .collect::<Vec<&str>>();  // Collect into a vector

    // Ensure repo_path has both owner and repo
    if repo_path.len() < 2 {
        bail!("Invalid REPO_URL format, must be <owner>/<repo>.");
    }

    // Format as <owner>/<repo>
    Ok(format!("{}/{}", repo_path[1], repo_path[0]))
}

/// Create a GitHub pull request using environment variables for configuration.
///
/// # Parameters:
/// - `req_client`: The `reqwest::Client` to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
/// - `head_branch`: The branch where changes are made.
/// - `base_branch`: The base branch to which the pull request is opened.
/// - `pr_title`: The title for the pull request.
///
/// The GitHub repository is pulled from the environment variable `REPO_NAME` in the `.env` file.
/// If the environment variable is not found, the function will return an error.
pub async fn create_pull_request(
    req_client: &Client,
    token: &str,
    head_branch: &str,
    base_branch: &str,
    pr_title: &str,
    pr_description: &str,
) -> Result<String> {
    // Get the repository name
    let repo_name = get_repo_name_from_env()?;

    // Prepare the JSON body for the pull request
    let pr_body = json!( {
        "title": pr_title,
        "head": head_branch,  // The branch with the changes
        "base": base_branch,  // The branch to merge into
        "body": pr_description,
    });

    info!("Creating pull request to {}/repos/{}/pulls", GITHUB_API_URL, repo_name);

    // Send the pull request creation request to the GitHub API
    let response = req_client
        .post(format!("{}/repos/{}/pulls", GITHUB_API_URL, repo_name))
        .bearer_auth(token)  // Use the GitHub access token for authentication
        .header("User-Agent", "Hyde")  // Set the User-Agent header to the app name
        .json(&pr_body)  // Include the PR body as JSON
        .send()
        .await?;

    // Handle the response based on the status code
    if response.status().is_success() {
        info!("Pull request created successfully for branch {}", head_branch);
        
        // Extract the response JSON to get the pull request URL
        let response_json: Value = response.json().await?;
        if let Some(url) = response_json.get("html_url").and_then(Value::as_str) {
            Ok(url.to_string()) // Directly return the URL as String
        } else {
            bail!("Expected URL field not found in the response.");
        }
    } else {
        let status = response.status();
        let response_text = response.text().await?;
        bail!(
            "Failed to create pull request: {}, Response: {}",
            status,
            response_text
        );
    }
}

/// Fetch a list of branches from the GitHub repository.
///
/// # Parameters:
/// - `req_client`: The `reqwest::Client` to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
/// - `repo_name`: The repository name in the format `<owner>/<repo>`.
///
/// # Returns:
/// A `Result` containing a vector of branch names or an error.
pub async fn list_branches(req_client: &Client, token: &str) -> Result<Vec<Branch>> {
    let repo_name = get_repo_name_from_env()?;
    let response = req_client
        .get(format!("{}/repos/{}/branches", GITHUB_API_URL, repo_name))
        .bearer_auth(token)
        .header("User-Agent", "Hyde")
        .send()
        .await?;

    if response.status().is_success() {
        info!("Branches fetched successfully for repository: {}", repo_name);
        let response_text = response.text().await?;
        
        // Attempt deserialization with detailed error handling
        let branches: Vec<Branch> = match serde_json::from_str(&response_text) {
            Ok(branches) => branches,
            Err(err) => {
                error!("Failed to deserialize branches: {}", err);
                return Err(err.into()); // Return error or handle it as needed
            }
        };

        Ok(branches)
    } else {
        let status = response.status();
        let response_text = response.text().await?;
        bail!(
            "Failed to fetch branches: {}, Response: {}",
            status,
            response_text
        );
    }
}

/// Fetch detailed information about a specific branch.
/// 
/// # Parameters:
/// - `req_client`: The `reqwest::Client` to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
/// - `branch_name`: The name of the branch to fetch details for.
/// 
/// # Returns:
/// A `Result` containing the branch details or an error.
async fn get_branch_details(req_client: &Client, token: &str, branch_name: &str) -> Result<Branch> {
    let repo_name = get_repo_name_from_env()?;
    let response = req_client
        .get(format!("{}/repos/{}/branches/{}", GITHUB_API_URL, repo_name, branch_name))
        .bearer_auth(token)
        .header("User-Agent", "Hyde")
        .send()
        .await?;

    if response.status().is_success() {
        info!("Fetched details for branch: {}", branch_name);
        let branch_details: Branch = response.json().await?;
        Ok(branch_details)
    } else {
        let status = response.status();
        let response_text = response.text().await?;
        bail!(
            "Failed to fetch branch details: {}, Response: {}",
            status,
            response_text
        );
    }
}

/// Get details about all branches including whether they are protected or the default branch.
pub async fn get_all_branch_details(req_client: &Client, token: &str) -> Result<Vec<Branch>> {
    let branches = list_branches(req_client, token).await?;
    let mut branch_details = Vec::new();

    for branch in branches {
        let details = get_branch_details(req_client, token, &branch.name).await?;
        branch_details.push(details);
    }

    Ok(branch_details)
}
