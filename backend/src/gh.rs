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
use tracing::{ info, error, debug };

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

pub struct GitHubClient {
    repo_url: String,
    client: Client,
    token: String,
}

impl GitHubClient {
    /// Creates a new instance of `GitHubClient`.
    /// 
    /// # Arguments
    /// - `repo_url` - A `String` representing the URL of the GitHub repository.
    /// - `client` - A `reqwest::Client` used for making HTTP requests to GitHub's API.
    /// - `token` - A `String` representing the GitHub access token used for authentication.
    ///
    /// # Returns
    /// A new `GitHubClient` instance that can be used to interact with the GitHub API.
    pub const fn new(repo_url: String, client: Client, token: String) -> Self {
        Self { repo_url, client, token }
    }

    /// Extracts the repository name and owner from a GitHub repository URL in the format `<owner>/<repo>`.
    /// 
    /// This function expects the `repo_url` to be in the format `https://<host>/<owner>/<repo>.git` (e.g., 
    /// `https://github.com/owner/repository.git`). It removes the `.git` suffix and extracts the owner 
    /// and repository name. The result is returned as a string in the format `<owner>/<repo>`.
    /// 
    /// # Returns
    /// A `Result<String>`, where:
    /// - `Ok(<owner>/<repo>)`: A string in the format `<owner>/<repo>`, representing the repository owner 
    ///   and name extracted from the URL.
    /// - `Err(e)`: An error message if the URL is not in the expected format or missing the `.git` suffix.
    /// 
    /// # Errors
    /// This function returns an error if:
    /// - The URL does not contain both an owner and a repository name (e.g., `https://github.com`).
    /// - The URL does not match the expected pattern (missing or incorrect `.git` suffix).
    fn get_repo_name(&self) -> Result<String> {
        let repo_path = self
            .repo_url
            .trim_end_matches(".git")
            .rsplit('/')
            .collect::<Vec<&str>>();

        if repo_path.len() < 2 {
            bail!("Invalid repo_url format, must be <owner>/<repo>.");
        }

        Ok(format!("{}/{}", repo_path[1], repo_path[0]))
    }

    /// Creates a GitHub pull request using the provided parameters.
    /// 
    /// This function sends a request to the GitHub API to create a pull request from a specified head branch 
    /// to a base branch in the specified repository. It requires the repository URL, an authentication token, 
    /// and the details of the pull request (title, description, and branches involved).
    /// 
    /// # Parameters:
    /// - `head_branch`: A string slice representing the branch with changes (source branch).
    /// - `base_branch`: A string slice representing the base branch to which the pull request is created (target branch).
    /// - `pr_title`: A string slice representing the title of the pull request.
    /// - `pr_description`: A string slice representing the description of the pull request.
    /// 
    /// # Returns:
    /// A `Result<String>`:
    /// - `Ok(url)`: If the pull request is successfully created, it returns the URL of the created pull request.
    /// - `Err(e)`: If the pull request creation fails, it returns an error message describing the failure.
    /// 
    /// # Errors:
    /// This function may return an error if:
    /// - The `repo_url` is not in the expected format and cannot be parsed to derive the repository name.
    /// - The request to create the pull request fails due to authentication issues, invalid input, or network problems.
    /// - The GitHub API response is missing the expected `html_url` field for the created pull request.
    pub async fn create_pull_request(
        &self,
        head_branch: &str,
        base_branch: &str,
        pr_title: &str,
        pr_description: &str,
        issue_numbers: Option<Vec<u64>>,
    ) -> Result<String> {
        // Parse the repository name from self.repo_url
        let repo_name = self.get_repo_name()?;

        let mut pr_body = pr_description.to_string();

        // If issue numbers are provided, add them to the body
        if let Some(issues) = issue_numbers {
            for issue in issues {
                pr_body.push_str(&format!("\n\nCloses #{}", issue)); // Add "Closes #<issue_number>" for each issue
            }
        }

        let pr_body_json = json!({
            "title": pr_title,
            "head": head_branch,
            "base": base_branch,
            "body": pr_body,
        });

        debug!("Creating pull request to {}/repos/{}/pulls", GITHUB_API_URL, repo_name);

        // Send the pull request creation request to the GitHub API
        let response = self
            .client
            .post(format!("{}/repos/{}/pulls", GITHUB_API_URL, repo_name))
            .bearer_auth(&self.token)
            .header("User-Agent", "Hyde")
            .json(&pr_body_json)
            .send()
            .await?;

        // Handle the response based on the status code
        if response.status().is_success() {
            info!(
                "Pull request created to merge {} into {}",
                head_branch, base_branch
            );
            
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

    pub async fn update_pull_request(
        &self,
        pr_number: u64,
        pr_title: Option<&str>,
        pr_description: Option<&str>,
        base_branch: Option<&str>,
        issue_numbers: Option<Vec<u64>>, // Add this parameter to pass the issues
    ) -> Result<String> {
        info!("Made it to the start of the update PR");
        let repo_name = self.get_repo_name()?;
    
        let mut pr_body_json = serde_json::Map::new();
    
        if let Some(title) = pr_title {
            pr_body_json.insert("title".to_string(), json!(title));
        }
    
        let mut pr_body = String::new();
    
        // If description is provided, include it in the body
        if let Some(description) = pr_description {
            pr_body.push_str(description);
        }
    
        // If issue numbers are provided, add them to the body
        if let Some(issues) = issue_numbers {
            for issue in issues {
                pr_body.push_str(&format!("\n\nCloses #{}", issue)); // Add "Closes #<issue_number>" for each issue
            }
        }
    
        // Add the constructed body to the JSON body
        pr_body_json.insert("body".to_string(), json!(pr_body));
    
        if let Some(base) = base_branch {
            pr_body_json.insert("base".to_string(), json!(base));
        }
    
        debug!("Updating pull request {} in {}/repos/{}/pulls", pr_number, GITHUB_API_URL, repo_name);
    
        // Send the request to the GitHub API to update the pull request
        let response = self
            .client
            .patch(format!("{}/repos/{}/pulls/{}", GITHUB_API_URL, repo_name, pr_number))
            .bearer_auth(&self.token)
            .header("User-Agent", "Hyde")
            .json(&pr_body_json)
            .send()
            .await?;
    
        // Handle the response based on the status code
        if response.status().is_success() {
            info!("Pull request #{} updated successfully", pr_number);
    
            // Extract the response JSON to get the updated pull request URL
            let response_json: Value = response.json().await?;
            if let Some(url) = response_json.get("html_url").and_then(Value::as_str) {
                Ok(url.to_string()) // Return the updated URL
            } else {
                bail!("Expected URL field not found in the response.");
            }
        } else {
            let status = response.status();
            let response_text = response.text().await?;
            bail!(
                "Failed to update pull request #{}: {}, Response: {}",
                pr_number,
                status,
                response_text
            );
        }
    }

    /// Fetches a complete list of branches from the specified GitHub repository.
    ///
    /// This function retrieves all branches for a repository by sending paginated GET requests to the GitHub API.
    /// It uses the repository name (extracted internally) to make authenticated requests using a GitHub token.
    /// The function iterates over pages of results, each containing up to 100 branches, until it has fetched all branches.
    /// The responses are deserialized into a vector of `Branch` structs.
    ///
    /// # Returns:
    /// A `Result<Vec<Branch>>`:
    /// - `Ok(branches)`: A vector of `Branch` structs representing all branches in the repository.
    /// - `Err(e)`: An error if the request fails or if the response cannot be deserialized into `Branch` structs.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The request to fetch branches fails (e.g., due to network issues, authentication errors, or API rate limits).
    /// - The response from the GitHub API cannot be deserialized into a vector of `Branch` structs.
    ///
    /// # Pagination:
    /// GitHub API paginates branch lists with a default limit of 30 branches per page. This function specifies a
    /// `per_page` limit of 100 branches to reduce the number of requests. It continues to fetch pages until no
    /// branches are left, ensuring that all branches are retrieved.
    pub async fn list_branches(&self) -> Result<Vec<Branch>> {
        let repo_name = self.get_repo_name()?;
        let mut branches = Vec::new();
        let mut page = 1;
    
        loop {
            // Make a GET request to fetch a page of branches
            let response = self
                .client
                .get(format!(
                    "{}/repos/{}/branches",
                    GITHUB_API_URL, repo_name
                ))
                .bearer_auth(&self.token)
                .header("User-Agent", "Hyde")
                .query(&[("per_page", "100"), ("page", &page.to_string())])
                .send()
                .await?;
    
            // Check response status and handle it accordingly
            if response.status().is_success() {
                let response_text = response.text().await?;
                let page_branches: Vec<Branch> = match serde_json::from_str(&response_text) {
                    Ok(branches) => branches,
                    Err(err) => {
                        error!("Failed to deserialize branches: {}", err);
                        return Err(err.into());
                    }
                };
    
                if page_branches.is_empty() {
                    break;
                }
    
                branches.extend(page_branches);
                page += 1;
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
    
        Ok(branches)
    }

    /// Fetches detailed information about a specific branch from a GitHub repository.
    ///
    /// This function sends a request to the GitHub API to retrieve detailed information about a
    /// specific branch, including metadata like the commit history, if available.
    ///
    /// # Parameters:
    /// - `branch_name`: The name of the branch for which to fetch details. This is a required parameter
    ///   to specify which branch to fetch information about.
    ///
    /// # Returns:
    /// - `Ok(Branch)`: A `Branch` struct containing detailed information about the specified branch.
    /// - `Err(e)`: An error if the request fails for reasons such as invalid input, authentication failure,
    ///   or an issue with retrieving branch details.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The request to fetch branch details fails, for instance due to network issues or an invalid branch name.
    /// - The response from the GitHub API cannot be deserialized into a `Branch` struct due to unexpected format
    ///   or missing data.
    pub async fn get_branch_details(&self, branch_name: &str) -> Result<Branch> {
        // Extract repository name from `repo_url`
        let repo_name = self.get_repo_name()?;

        // Send the request to get branch details for the specified branch name
        let response = self
            .client
            .get(format!("{}/repos/{}/branches/{}", GITHUB_API_URL, repo_name, branch_name))
            .bearer_auth(&self.token)
            .header("User-Agent", "Hyde")
            .send()
            .await?;

        // Handle the response
        if response.status().is_success() {
            // Deserialize the JSON response to a Branch struct
            let branch_details: Branch = response.json().await?;
            Ok(branch_details)
        } else {
            // Handle errors with detailed information
            let status = response.status();
            let response_text = response.text().await?;
            bail!(
                "Failed to fetch branch details: {}, Response: {}",
                status,
                response_text
            );
        }
    }

    /// Get details about all branches, including whether they are protected or the default branch.
    ///
    /// This function fetches the list of branches from the repository and then retrieves detailed
    /// information about each branch, such as its protection status and whether it is the default branch.
    ///
    /// # Returns:
    /// A `Result<Vec<Branch>>` containing a vector of `Branch` structs with detailed information about each
    /// branch if successful, or an error if the request to fetch branch details fails.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The request to fetch the list of branches fails.
    /// - The request to fetch details for any individual branch fails.
    pub async fn get_all_branch_details(&self) -> Result<Vec<Branch>> {
        // Get a list of branches
        let branches = self.list_branches().await?;
        let mut branch_details = Vec::new();

        // Fetch details for each branch
        for branch in branches {
            let details = self.get_branch_details(&branch.name).await?;
            branch_details.push(details);
        }

        Ok(branch_details)
    }

    /// Fetches issues from the GitHub repository.
    ///
    /// This function retrieves issues from the specified repository using the GitHub API.
    /// You can filter issues based on their state and associated labels.
    ///
    /// # Parameters:
    /// - `state`: A string slice representing the state of the issues to fetch (e.g., "open", "closed", "all").
    ///            Defaults to "open".
    /// - `labels`: A comma-separated string slice representing labels to filter issues by. Defaults to `None`.
    ///
    /// # Returns:
    /// A `Result<Vec<Value>>`:
    /// - `Ok(issues)`: A vector of JSON values representing the issues fetched from the repository.
    /// - `Err(e)`: An error message if the request fails or the response cannot be parsed.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The `repo_url` is not in the expected format and cannot be parsed to derive the repository name.
    /// - The request to fetch issues fails due to authentication issues, invalid input, or network problems.
    /// - The GitHub API response cannot be parsed as a JSON array.
    pub async fn get_issues(&self, state: Option<&str>, labels: Option<&str>) -> Result<Vec<Value>> {
        let repo_name = self.get_repo_name().map_err(|e| {
            error!("Failed to get repository name: {:?}", e);
            e
        })?;
    
        let state = state.unwrap_or("open"); // Default state
        let mut query_params = vec![format!("state={}", state)];
        if let Some(labels) = labels {
            query_params.push(format!("labels={}", labels));
        }
        let query_string = format!("?{}", query_params.join("&"));
    
        let url = format!("{}/repos/{}/issues{}", GITHUB_API_URL, repo_name, query_string);
        debug!("Request URL: {}", url);
    
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Hyde")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
    
        if let Some(rate_limit_remaining) = response.headers().get("X-RateLimit-Remaining") {
            debug!("GitHub API rate limit remaining: {}", rate_limit_remaining.to_str().unwrap_or("Unknown"));
        }
    
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("GitHub API request failed with status {}: {}", status, error_text);
            bail!("GitHub API request failed ({}): {}", status, error_text);
        }
    
        let issues: Vec<Value> = response.json().await.map_err(|e| {
            error!("Failed to parse GitHub response JSON: {:?}", e);
            e
        })?;
    
        Ok(issues)
    }                

}
