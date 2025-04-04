//! Code for interacting with GitHub (authentication, pull requests, et cetera)

use chrono::DateTime;
use color_eyre::Result;
use color_eyre::eyre::{Context, bail};
use fs_err as fs;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::io::Read;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, info};

const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Clone)]
pub struct GitHubClient {
    /// The URL of the GitHub repository this client is associated with.
    repo_url: String,
    /// An HTTP client used to make requests to the GitHub API.
    client: Client,
    /// The client ID for GitHub OAuth authentication.
    client_id: String,
    /// A thread-safe, shared access token for authenticating requests.
    token: Arc<Mutex<String>>,
    /// The expiration time of the current authentication token.
    expires_at: Arc<Mutex<SystemTime>>,
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
    /// - A new `GitHubClient` instance that can be used to interact with the GitHub API.
    pub fn new(repo_url: String, client: Client, client_id: String) -> Self {
        Self {
            repo_url,
            client,
            client_id,
            token: Arc::new(Mutex::new(String::new())),
            expires_at: Arc::new(Mutex::new(UNIX_EPOCH)),
        }
    }

    /// Retrieves a valid GitHub access token, refreshing it if necessary.
    ///
    /// This function ensures that a valid access token is always available for making authenticated
    /// requests to the GitHub API. If the current token is close to expiration or unavailable,
    /// the function fetches a new token using the provided client credentials and updates the token
    /// and expiration time.
    ///
    /// # Returns:
    /// A `Result<String>`:
    /// - `Ok(String)`: A valid access token as a `String`.
    /// - `Err(e)`: An error if the token retrieval or refresh process fails.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The current time cannot be determined (`SystemTime` issues).
    /// - The token refresh request to the GitHub API fails.
    /// - The response from the token refresh endpoint cannot be parsed or does not contain valid token data.
    ///
    pub async fn get_token(&self) -> Result<String> {
        let mut token_ref = self.token.lock().await;

        // Fetch a new token if more than 59 minutes have passed
        // Tokens expire after 1 hour, this is to account for clock drift
        if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() > (60 * 59) {
            // Fetch a new token
            let api_response = self.get_access_token().await?;
            *token_ref = api_response.0;
            let mut expires_ref = self.expires_at.lock().await;
            *expires_ref = api_response.1;
        }

        Ok(token_ref.clone())
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
    #[tracing::instrument(level = "debug", skip(self))]
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
    #[tracing::instrument(level = "debug", skip(self))]
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
        let token = self.get_token().await?;
        let mut pr_body = pr_description.to_string();

        // If issue numbers are provided, add them to the body
        if let Some(issues) = issue_numbers {
            for issue in issues {
                pr_body.push_str(&format!("\n\nCloses #{}", issue));
            }
        }

        let pr_body_json = json!({
            "title": pr_title,
            "head": head_branch,
            "base": base_branch,
            "body": pr_body,
        });

        debug!(
            "Creating pull request to {}/repos/{}/pulls",
            GITHUB_API_URL, repo_name
        );

        // Send the pull request creation request to the GitHub API
        let response = self
            .client
            .post(format!("{}/repos/{}/pulls", GITHUB_API_URL, repo_name))
            .bearer_auth(&token)
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
                Ok(url.to_string())
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

    /// Updates an existing pull request on GitHub with the specified details.
    ///
    /// This function sends a `PATCH` request to the GitHub API to update an existing pull request.
    /// It allows updating the title, description, base branch, and associated issues of the pull request.
    ///
    /// # Arguments
    /// - `pr_number` - The number of the pull request to update.
    /// - `pr_title` - Optional new title for the pull request.
    /// - `pr_description` - Optional updated description for the pull request.
    /// - `base_branch` - Optional target base branch to change the pull request's target.
    /// - `issue_numbers` - Optional list of GitHub issue numbers to associate with the pull request.
    ///   These issues will be referenced in the pull request description using the "Closes #<issue_number>" syntax.
    ///
    /// # Returns
    /// A `Result<String>`:
    /// - `Ok(url)`: The URL of the updated pull request if the operation is successful.
    /// - `Err(e)`: An error if the operation fails.
    ///
    /// # Errors
    /// This function returns an error in the following scenarios:
    /// - The repository name cannot be retrieved.
    /// - The GitHub API request fails due to network issues, authentication problems, or a bad request.
    /// - The response from GitHub does not contain the expected `html_url` field, indicating an unexpected API response format.
    /// - The request body cannot be constructed due to missing or invalid arguments.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn update_pull_request(
        &self,
        pr_number: u64,
        pr_title: Option<&str>,
        pr_description: Option<&str>,
        base_branch: Option<&str>,
        issue_numbers: Option<Vec<u64>>,
    ) -> Result<String> {
        let repo_name = self.get_repo_name()?;
        let token = self.get_token().await?;
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
                pr_body.push_str(&format!("\n\nCloses #{}", issue));
            }
        }

        // Add the constructed body to the JSON body
        pr_body_json.insert("body".to_string(), json!(pr_body));

        if let Some(base) = base_branch {
            pr_body_json.insert("base".to_string(), json!(base));
        }

        debug!(
            "Updating pull request {} in {}/repos/{}/pulls",
            pr_number, GITHUB_API_URL, repo_name
        );

        // Send the request to the GitHub API to update the pull request
        let response = self
            .client
            .patch(format!(
                "{}/repos/{}/pulls/{}",
                GITHUB_API_URL, repo_name, pr_number
            ))
            .bearer_auth(&token)
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
                Ok(url.to_string())
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

    /// Closes a pull request in the specified GitHub repository.
    ///
    /// This function sends a `PATCH` request to the GitHub API to change the state
    /// of a pull request to "closed". It requires the repository's name and a valid
    /// authentication token to authenticate and send the request.
    ///
    /// # Arguments
    /// - `pr_number`: The number of the pull request to close.
    ///
    /// # Returns
    /// A `Result<()>`:
    /// - `Ok(())` if the pull request was successfully closed.
    /// - `Err(e)` if an error occurred during the process, such as:
    ///   - Issues with fetching the repository name.
    ///   - Failure to acquire a valid authentication token.
    ///   - Network issues or problems with sending the request to the GitHub API.
    ///
    /// # Errors
    /// This function returns an error in the following cases:
    /// - The repository name cannot be fetched from the GitHub client.
    /// - The token required for authentication cannot be obtained or is invalid.
    /// - The GitHub API request fails (e.g., due to incorrect repository or pull request numbers).
    /// - The response from GitHub does not match the expected status for closing the pull request.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn close_pull_request(&self, pr_number: u64) -> Result<()> {
        // Get the repository name from the repository URL
        let repo_name = self.get_repo_name()?;
        let token = self.get_token().await?;

        // Construct the JSON body to close the pull request
        let pr_body_json = json!({
            "state": "closed"
        });

        // Send the request to GitHub API to close the pull request
        let response = self
            .client
            .patch(format!(
                "{}/repos/{}/pulls/{}",
                GITHUB_API_URL, repo_name, pr_number
            ))
            .bearer_auth(&token)
            .header("User-Agent", "Hyde")
            .json(&pr_body_json)
            .send()
            .await?;

        // Handle the response
        if response.status().is_success() {
            info!("Pull request #{} closed successfully", pr_number);
            Ok(())
        } else {
            let status = response.status();
            let response_text = response.text().await?;
            bail!(
                "Failed to close pull request #{}: {}, Response: {}",
                pr_number,
                status,
                response_text
            );
        }
    }

    /// Fetches a complete list of branches with detailed information from the specified GitHub repository.
    ///
    /// This function retrieves all branches for a repository by sending paginated GET requests to the GitHub API.
    /// Each response includes detailed information about each branch, such as whether it is protected and its commit metadata.
    ///
    /// # Returns:
    /// A `Result<Vec<Branch>>`:
    /// - `Ok(branches)`: A vector of `Branch` structs representing all branches in the repository, including detailed information.
    /// - `Err(e)`: An error if the request fails or if the response cannot be deserialized into `Branch` structs.
    ///
    /// # Errors:
    /// This function may return an error if:
    /// - The request to fetch branches fails (e.g., due to network issues, authentication errors, or API rate limits).
    /// - The response from the GitHub API cannot be deserialized into a vector of `Branch` structs.
    ///
    /// # Pagination:
    /// The GitHub API paginates branch lists with a default limit of 30 branches per page. This function specifies a
    /// `per_page` limit of 100 branches to reduce the number of requests. It continues to fetch pages until no
    /// branches are left, ensuring that all branches are retrieved.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn list_branches(&self) -> Result<Vec<Branch>> {
        let repo_name = self.get_repo_name()?;
        let token = self.get_token().await?;
        let mut branches = Vec::new();
        let mut page = 1;

        loop {
            // Make a GET request to fetch a page of branches
            let response = self
                .client
                .get(format!("{}/repos/{}/branches", GITHUB_API_URL, repo_name))
                .bearer_auth(&token)
                .header("User-Agent", "Hyde")
                .query(&[("per_page", "100"), ("page", &page.to_string())])
                .send()
                .await?;

            // Check response status and handle it accordingly
            if response.status().is_success() {
                let page_branches: Vec<Branch> = response.json().await?;

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

    /// Fetches the default branch of the repository associated with the authenticated user.
    ///
    /// This function retrieves the repository name using `get_repo_name`,
    /// sends a `GET` request to the GitHub API to fetch repository details,
    /// and extracts the default branch from the response.
    ///
    /// # Returns
    /// A `Result<String>`:
    /// - `Ok(String)` containing the name of the default branch if successful.
    /// - `Err(anyhow::Error)` if an error occurs during the process, such as:
    ///   - Failure to retrieve the repository name.
    ///   - Issues with sending the `GET` request or network problems.
    ///   - Failure to parse the response or if the `default_branch` field is missing from the response.
    ///
    /// # Errors
    /// Returns an error in the following cases:
    /// - The repository name cannot be retrieved from the GitHub client.
    /// - The `GET` request to fetch repository details fails (e.g., due to network issues or API errors).
    /// - The response from GitHub does not contain a valid `default_branch` field.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_default_branch(&self) -> Result<String> {
        // Extract repository name from `repo_url`
        let repo_name = self.get_repo_name()?;
        let token = self.get_token().await?;

        // Make the GET request to fetch repository details
        let response = self
            .client
            .get(format!("{}/repos/{}", GITHUB_API_URL, repo_name))
            .bearer_auth(&token)
            .header("User-Agent", "Hyde")
            .send()
            .await?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let response_text = response.text().await?;
            bail!(
                "Failed to fetch repository details: {}, Response: {}",
                status,
                response_text
            );
        }

        // Deserialize the response to get the repository details
        let repo_details: Map<String, Value> = response.json().await?;

        // Retrieve the default branch from the response
        let serialized_default_branch = repo_details
            .get("default_branch")
            .expect("GitHub API response missing expected field 'default_branch'");
        let default_branch = serialized_default_branch.as_str().unwrap().to_owned();

        Ok(default_branch)
    }

    /// Fetches issues from the GitHub repository.
    ///
    /// This function retrieves issues from the specified repository using the GitHub API.
    /// You can filter issues based on their issue_state and associated labels.
    ///
    /// # Parameters:
    /// - `issue_state`: A string slice representing the issue_state of the issues to fetch (e.g., "open", "closed", "all").
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_issues(
        &self,
        state: Option<&str>,
        labels: Option<&str>,
    ) -> Result<Vec<Value>> {
        let repo_name = self.get_repo_name()?;

        let issue_state = state.unwrap_or("open");
        let token = self.get_token().await?;
        let mut query_params = vec![format!("state={}", issue_state)];
        if let Some(labels) = labels {
            query_params.push(format!("labels={}", labels));
        }
        let query_string = format!("?{}", query_params.join("&"));

        let url = format!(
            "{}/repos/{}/issues{}",
            GITHUB_API_URL, repo_name, query_string
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Hyde")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            bail!("GitHub API request failed ({}): {}", status, error_text);
        }

        let issues: Vec<Value> = response.json().await?;

        Ok(issues)
    }

    /// Request a github installation access token using the provided reqwest client.
    /// The installation access token will expire after 1 hour.
    /// Returns the new token, and the time of expiration
    async fn get_access_token(&self) -> Result<(String, SystemTime)> {
        let token = self.gen_jwt_token()?;
        let response = self
            .client
            .post(format!(
                "https://api.github.com/app/installations/{}/access_tokens",
                self.get_installation_id().await?
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

    /// Fetch the Installation ID. This value is required for most API calls
    ///
    /// <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation#generating-an-installation-access-token>
    async fn get_installation_id(&self) -> Result<String> {
        let response = self
            .client
            .get("https://api.github.com/app/installations")
            .bearer_auth(self.gen_jwt_token()?)
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
    fn gen_jwt_token(&self) -> Result<String> {
        let mut private_key_file = fs::File::open("hyde-data/key.pem")
            .wrap_err("Failed to read private key from `hyde-data/key.pem`")?;
        let mut private_key = Vec::new();
        private_key_file.read_to_end(&mut private_key)?;
        Ok(encode(
            &Header::new(Algorithm::RS256),
            &Claims::new(&self.client_id)?,
            &EncodingKey::from_rsa_pem(&private_key)?,
        )?)
    }
}

/// In order to authenticate as a Github app or generate an installation access token, you must generate a JSON Web Token (JWT). The JWT must contain predefined *claims*.
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

#[derive(Deserialize)]
struct AccessTokenResponse {
    expires_at: String,
    token: String,
}

#[derive(Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub protected: bool,
}

#[derive(Deserialize)]
struct InstallationIdResponse {
    id: u64,
}
