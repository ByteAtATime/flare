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
    Initialize {
        preferences: std::collections::HashMap<String, serde_json::Value>,
    },
    InvokeCallback {
        callback_id: String,
        args: serde_json::Value,
    },
    Pop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RustResponse {
    Success {
        id: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
    Error {
        id: u32,
        error: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
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
        tree: Tree,
    },
    CacheSet {
        id: u32,
        namespace: String,
        key: String,
        data: String,
    },
    CacheGet {
        id: u32,
        namespace: String,
        key: String,
    },
    CacheHas {
        id: u32,
        namespace: String,
        key: String,
    },
    CacheRemove {
        id: u32,
        namespace: String,
        key: String,
    },
    CacheClear {
        id: u32,
        namespace: String,
    },
    CacheIsEmpty {
        id: u32,
        namespace: String,
    },
    Pop {
        id: u32,
    },
}
