use super::{types::*, GitHubClient};

impl GitHubClient {
    /// Search for issues and pull requests
    pub async fn search_issues(&self, query: &str) -> Result<SearchResult<Issue>, String> {
        let path = format!("/search/issues?q={}", urlencoding::encode(query));

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to search issues: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse search results: {}", e))
    }

    /// Search for repositories
    pub async fn search_repos(&self, query: &str) -> Result<SearchResult<Repository>, String> {
        let path = format!("/search/repositories?q={}", urlencoding::encode(query));

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to search repositories: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse search results: {}", e))
    }

    /// List repositories for the authenticated user
    pub async fn list_user_repos(&self) -> Result<Vec<Repository>, String> {
        let path = "/user/repos?per_page=100&sort=updated";

        let response = self
            .build_request(reqwest::Method::GET, path)
            .send()
            .await
            .map_err(|e| format!("Failed to list repositories: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse repositories response: {}", e))
    }

    /// Get a specific repository
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository, String> {
        let path = format!("/repos/{}/{}", owner, repo);

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to get repository: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse repository response: {}", e))
    }
}
