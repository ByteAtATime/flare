use super::{types::*, GitHubClient};

impl GitHubClient {
    /// List issues for a repository
    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<Issue>, String> {
        let mut path = format!("/repos/{}/{}/issues", owner, repo);

        if let Some(state) = state {
            path.push_str(&format!("?state={}", state));
        }

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to list issues: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse issues response: {}", e))
    }

    /// Get a specific issue
    pub async fn get_issue(&self, owner: &str, repo: &str, number: u64) -> Result<Issue, String> {
        let path = format!("/repos/{}/{}/issues/{}", owner, repo, number);

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to get issue: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse issue response: {}", e))
    }

    /// Create a new issue
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: String,
        body: Option<String>,
        labels: Option<Vec<String>>,
        assignees: Option<Vec<String>>,
    ) -> Result<Issue, String> {
        let path = format!("/repos/{}/{}/issues", owner, repo);

        let mut payload = serde_json::json!({
            "title": title,
        });

        if let Some(body) = body {
            payload["body"] = serde_json::json!(body);
        }

        if let Some(labels) = labels {
            payload["labels"] = serde_json::json!(labels);
        }

        if let Some(assignees) = assignees {
            payload["assignees"] = serde_json::json!(assignees);
        }

        let response = self
            .build_request(reqwest::Method::POST, &path)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to create issue: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error {}: {}", status, error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse created issue response: {}", e))
    }

    /// Update an existing issue
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        title: Option<String>,
        body: Option<String>,
        state: Option<&str>,
        labels: Option<Vec<String>>,
        assignees: Option<Vec<String>>,
    ) -> Result<Issue, String> {
        let path = format!("/repos/{}/{}/issues/{}", owner, repo, number);

        let mut payload = serde_json::json!({});

        if let Some(title) = title {
            payload["title"] = serde_json::json!(title);
        }

        if let Some(body) = body {
            payload["body"] = serde_json::json!(body);
        }

        if let Some(state) = state {
            payload["state"] = serde_json::json!(state);
        }

        if let Some(labels) = labels {
            payload["labels"] = serde_json::json!(labels);
        }

        if let Some(assignees) = assignees {
            payload["assignees"] = serde_json::json!(assignees);
        }

        let response = self
            .build_request(reqwest::Method::PATCH, &path)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to update issue: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse updated issue response: {}", e))
    }

    /// Close an issue
    pub async fn close_issue(&self, owner: &str, repo: &str, number: u64) -> Result<Issue, String> {
        self.update_issue(owner, repo, number, None, None, Some("closed"), None, None)
            .await
    }

    /// List issues assigned to the authenticated user
    pub async fn list_my_issues(&self, state: Option<&str>) -> Result<Vec<Issue>, String> {
        let mut path = "/issues".to_string();

        if let Some(state) = state {
            path.push_str(&format!("?state={}", state));
        }

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .map_err(|e| format!("Failed to list my issues: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse my issues response: {}", e))
    }
}
