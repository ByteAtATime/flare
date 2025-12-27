use serde::{Deserialize, Serialize};
use std::time::Duration;

const GITHUB_CLIENT_ID: &str = "Ov23liLBXQcwvZPYjDGh"; // Flareup GitHub OAuth App
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
#[allow(dead_code)]
const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenResponse {
    Success {
        access_token: String,
        token_type: String,
        scope: String,
    },
    Pending {
        error: String,
        error_description: String,
    },
}

/// Start the OAuth device flow by requesting a device code
pub async fn start_device_flow() -> Result<DeviceCodeResponse, String> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", GITHUB_CLIENT_ID),
        ("scope", "repo user notifications"),
    ];

    let response = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request device code: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let device_code: DeviceCodeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse device code response: {}", e))?;

    Ok(device_code)
}

/// Poll for the access token using the device code
pub async fn poll_for_token(device_code: &str) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", GITHUB_CLIENT_ID),
        ("device_code", device_code),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];

    let response = client
        .post(ACCESS_TOKEN_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to poll for token: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    match token_response {
        TokenResponse::Success { access_token, .. } => Ok(Some(access_token)),
        TokenResponse::Pending { error, .. } => {
            if error == "authorization_pending" || error == "slow_down" {
                Ok(None) // Still waiting for user authorization
            } else if error == "expired_token" {
                Err(
                    "Device code expired. Please start the authentication process again."
                        .to_string(),
                )
            } else if error == "access_denied" {
                Err("User denied authorization.".to_string())
            } else {
                Err(format!("Authentication error: {}", error))
            }
        }
    }
}

/// Store the GitHub access token in the keyring
pub fn store_token(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("flareup", "github")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(token)
        .map_err(|e| format!("Failed to store token: {}", e))?;

    Ok(())
}

/// Retrieve the GitHub access token from the keyring
pub fn get_token() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new("flareup", "github")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve token: {}", e)),
    }
}

/// Delete the GitHub access token from the keyring
pub fn delete_token() -> Result<(), String> {
    let entry = keyring::Entry::new("flareup", "github")
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(format!("Failed to delete token: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_code_url() {
        assert_eq!(DEVICE_CODE_URL, "https://github.com/login/device/code");
    }

    #[test]
    fn test_client_id() {
        assert!(!GITHUB_CLIENT_ID.is_empty());
    }
}
