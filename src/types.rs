use crate::components::Component;
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct ToastOptions {
    pub title: String,
    pub message: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tree {
    pub id: String,
    pub children: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SidecarRequest {
    Initialize { plugin_path: String },
    InvokeCallback { callback_id: String, args: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SidecarResponse {
    Initialized { success: bool, error: Option<String> },
    CallbackResult { success: bool, error: Option<String> },
}
