use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub file_type: String, // "file" or "directory"
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at: String, // ISO 8601 timestamp
    pub accessed_at: Option<String>,
    pub is_complete: bool, // false if still downloading (.crdownload, .part)
}

/// File extensions that indicate an incomplete download
pub const INCOMPLETE_EXTENSIONS: &[&str] = &[
    "crdownload", // Chrome
    "part",       // Firefox, wget
    "download",   // Safari
    "tmp",        // Various
    "partial",    // Various
];

/// Check if a file extension indicates an incomplete download
pub fn is_incomplete_download(extension: Option<&str>) -> bool {
    extension.map_or(false, |ext| {
        INCOMPLETE_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    })
}
