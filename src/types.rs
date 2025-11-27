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
        assets_path: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClipboardContent {
    Text { text: String },
    File { file: String },
    Html { html: String, text: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardReadResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
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
    OpenExtensionPreferences {
        id: u32,
    },
    OpenCommandPreferences {
        id: u32,
    },
    LocalStorageSet {
        id: u32,
        namespace: String,
        key: String,
        data: String,
    },
    LocalStorageGet {
        id: u32,
        namespace: String,
        key: String,
    },
    LocalStorageRemove {
        id: u32,
        namespace: String,
        key: String,
    },
    LocalStorageClear {
        id: u32,
        namespace: String,
    },
    LocalStorageAll {
        id: u32,
        namespace: String,
    },
    ClipboardCopy {
        id: u32,
        content: ClipboardContent,
        concealed: bool,
    },
    ClipboardClear {
        id: u32,
    },
    ClipboardRead {
        id: u32,
        offset: Option<usize>,
    },
}
