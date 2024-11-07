//! Code for interacting with GitHub (authentication, prs, et cetera)

use chrono::DateTime;
use color_eyre::eyre::{bail, Context};
use color_eyre::Result;
use fs_err as fs;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    let mut private_key_file = fs::File::open("hyde-data/key.pem")
        .wrap_err("Failed to read private key from `hyde-data/key.pem`")?;
    let mut private_key = Vec::new();
    private_key_file.read_to_end(&mut private_key)?;
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &Claims::new(client_id)?,
        &EncodingKey::from_rsa_pem(&private_key)?,
    )?)
}

/// Extracts the repository name and owner from a GitHub repository URL in the format `<owner>/<repo>`.
/// 
/// This function expects the `repo_url` to be in the format `https://<host>/<owner>/<repo>.git` (e.g., 
/// `https://github.com/owner/repository.git`). It removes the `.git` suffix and extracts the owner 
/// and repository name. The result is returned as a string in the format `<owner>/<repo>`.
/// 
/// # Parameters
/// - `repo_url`: A string slice representing the full repository URL (e.g., 
///   `https://github.com/owner/repository.git`).
/// 
/// # Returns
/// A `Result<String>` where:
/// - `Ok(<owner>/<repo>)`: A string in the format `<owner>/<repo>` representing the repository name and owner.
/// - `Err(e)`: Returns an error if the `repo_url` is missing the expected format or `.git` suffix.
/// 
/// # Errors
/// Returns an error if:
/// - The URL does not contain both an owner and a repository name (e.g., `https://github.com`).
/// - The URL does not match the expected pattern (missing or incorrect `.git` suffix).
pub fn get_repo_name_from_url(repo_url: &str) -> Result<String> {
    // Parse the repository name from the URL
    let repo_path = repo_url
        .trim_end_matches(".git")  // Remove the .git suffix
        .rsplit('/')  // Split by '/'
        .collect::<Vec<&str>>();  // Collect into a vector

    // Ensure repo_path has both owner and repo
    if repo_path.len() < 2 {
        bail!("Invalid repo_url format, must be <owner>/<repo>.");
    }

    // Format as <owner>/<repo>
    Ok(format!("{}/{}", repo_path[1], repo_path[0]))
}

/// Create a GitHub pull request using the provided parameters.
///
/// # Parameters:
/// - `repo_url`: A string slice that contains the GitHub repository URL in the format
///   `https://<host>/<owner>/<repo>.git`. This is used to derive the repository name.
/// - `reqwest_client`: The `reqwest::Client` used to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
/// - `head_branch`: The branch where changes are made.
/// - `base_branch`: The base branch to which the pull request is opened.
/// - `pr_title`: The title for the pull request.
/// - `pr_description`: The description for the pull request.
///
/// # Returns
/// A `Result<String>` containing the URL of the created pull request if successful, or an error
/// if the pull request creation fails.
///
/// # Errors
/// Returns an error if:
/// - The `repo_url` is not in the expected format.
/// - The pull request creation request fails (e.g., due to authentication issues or invalid input).
pub async fn create_pull_request(
    repo_url: &str,
    reqwest_client: &reqwest::Client,
    token: &str,
    head_branch: &str,
    base_branch: &str,
    pr_title: &str,
    pr_description: &str,
) -> Result<String> {
    // Get the repository name using the updated function
    let repo_name = get_repo_name_from_url(repo_url)?;

    // Prepare the JSON body for the pull request
    let pr_body = json!( {
        "title": pr_title,
        "head": head_branch,  // The branch with the changes
        "base": base_branch,  // The branch to merge into
        "body": pr_description,
    });

    info!("Creating pull request to {}/repos/{}/pulls", GITHUB_API_URL, repo_name);

    // Send the pull request creation request to the GitHub API
    let response = reqwest_client
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

/// Fetch a list of branches from the specified GitHub repository.
///
/// # Parameters:
/// - `repo_url`: A string slice that contains the GitHub repository URL in the format
///   `https://<host>/<owner>/<repo>.git`. This is used to derive the repository name.
/// - `reqwest_client`: The `reqwest::Client` used to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
///
/// # Returns
/// A `Result<Vec<Branch>>` containing a vector of `Branch` structs if successful,
/// or an error if the request fails.
///
/// # Errors
/// Returns an error if:
/// - The `repo_url` is not in the expected format.
/// - The request to fetch branches fails.
pub async fn list_branches(
    repo_url: &str,
    reqwest_client: &reqwest::Client,
    token: &str,
) -> Result<Vec<Branch>> {
    // Get the repository name using the updated function
    let repo_name = get_repo_name_from_url(repo_url)?;

    let response = reqwest_client
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
/// - `repo_url`: A string slice that contains the GitHub repository URL in the format
///   `https://<host>/<owner>/<repo>.git`. This is used to derive the repository name.
/// - `reqwest_client`: The `reqwest::Client` used to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
/// - `branch_name`: The name of the branch to fetch details for.
///
/// # Returns:
/// A `Result<Branch>` containing the branch details if successful, or an error.
///
/// # Errors
/// Returns an error if:
/// - The `repo_url` is not in the expected format.
/// - The request to fetch branch details fails.
async fn get_branch_details(
    repo_url: &str,
    reqwest_client: &reqwest::Client,
    token: &str,
    branch_name: &str,
) -> Result<Branch> {
    // Get the repository name using the updated function
    let repo_name = get_repo_name_from_url(repo_url)?;

    let response = reqwest_client
        .get(format!("{}/repos/{}/branches/{}", GITHUB_API_URL, repo_name, branch_name))
        .bearer_auth(token)
        .header("User-Agent", "Hyde")
        .send()
        .await?;

    if response.status().is_success() {
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
///
/// # Parameters:
/// - `repo_url`: A string slice that contains the GitHub repository URL in the format
///   `https://<host>/<owner>/<repo>.git`. This is used to derive the repository name.
/// - `reqwest_client`: The `reqwest::Client` used to make HTTP requests.
/// - `token`: The GitHub access token for authentication.
///
/// # Returns:
/// A `Result<Vec<Branch>>` containing the details of all branches if successful, or an error.
///
/// # Errors
/// Returns an error if the request to fetch branch details fails.
pub async fn get_all_branch_details(
    repo_url: &str,
    reqwest_client: &reqwest::Client,
    token: &str,
) -> Result<Vec<Branch>> {
    let branches = list_branches(repo_url, reqwest_client, token).await?;
    let mut branch_details = Vec::new();

    for branch in branches {
        let details = get_branch_details(repo_url, reqwest_client, token, &branch.name).await?;
        branch_details.push(details);
    }

    Ok(branch_details)
}
