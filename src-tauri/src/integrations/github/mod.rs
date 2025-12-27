pub mod auth;
pub mod issues;
pub mod search;
pub mod types;

pub use auth::{
    delete_token, get_token, poll_for_token, start_device_flow, store_token, DeviceCodeResponse,
};
pub use types::*;

use reqwest::Client;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

pub struct GitHubClient {
    token: String,
    http_client: Client,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            http_client: Client::new(),
        }
    }

    /// Create a new client from stored token
    pub fn from_stored_token() -> Result<Self, String> {
        let token = get_token()?.ok_or("No GitHub token found. Please authenticate first.")?;
        Ok(Self::new(token))
    }

    /// Helper to build authenticated requests
    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", GITHUB_API_BASE, path);
        self.http_client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("User-Agent", "Flareup")
    }

    /// Test the authentication by getting the current user
    pub async fn get_current_user(&self) -> Result<User, String> {
        let response = self
            .build_request(reqwest::Method::GET, "/user")
            .send()
            .await
            .map_err(|e| format!("Failed to get current user: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user response: {}", e))
    }
}
