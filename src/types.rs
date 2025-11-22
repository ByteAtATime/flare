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
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SidecarRequest {
    InvokeCallback {
        callback_id: String,
        args: serde_json::Value,
    },
    Response {
        id: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SidecarResponse {
    Initialized {
        success: bool,
        error: Option<String>,
    },
    CallbackResult {
        success: bool,
        error: Option<String>,
    },
    ShowToast {
        id: u32,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<String>,
    },
    UpdateTree {
        id: u32,
        tree: serde_json::Value,
    },
    CacheSet {
        id: u32,
        namespace: String,
        key: String,
        data: String,
    },
}
