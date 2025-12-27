use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as RusqliteResult};
use tauri::{AppHandle, Manager};

use super::types::{is_incomplete_download, DownloadItem};
use crate::error::AppError;

pub struct DownloadsManager {
    db: Arc<Mutex<Connection>>,
}

impl DownloadsManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let data_dir = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|_| AppError::DirectoryNotFound)?;

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| AppError::FileSearch(e.to_string()))?;
        }

        let db_path = data_dir.join("downloads.sqlite");
        let db = Connection::open(db_path)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub fn init_db(&self) -> RusqliteResult<()> {
        let db = self.db.lock().expect("downloads db mutex poisoned");

        db.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                extension TEXT,
                file_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                accessed_at TEXT
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_created ON downloads(created_at DESC)",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_name ON downloads(name)",
            [],
        )?;

        Ok(())
    }

    pub fn add_download(&self, path: &Path) -> Result<Option<DownloadItem>, AppError> {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return Ok(None), // File doesn't exist or can't access
        };

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        // Skip incomplete downloads
        if is_incomplete_download(extension.as_deref()) {
            return Ok(None);
        }

        let file_type = if metadata.is_dir() {
            "directory"
        } else {
            "file"
        }
        .to_string();

        let size_bytes = metadata.len() as i64;

        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| DateTime::<Utc>::from(t).to_rfc3339())
            .unwrap_or_else(|_| Utc::now().to_rfc3339());

        let path_str = path.to_string_lossy().to_string();

        let db = self.db.lock().expect("downloads db mutex poisoned");
        db.execute(
            "INSERT OR REPLACE INTO downloads (path, name, extension, file_type, size_bytes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![path_str, name, extension, file_type, size_bytes, created_at],
        )?;

        let id = db.last_insert_rowid();

        Ok(Some(DownloadItem {
            id,
            path: path_str,
            name,
            file_type,
            extension,
            size_bytes,
            created_at,
            accessed_at: None,
            is_complete: true,
        }))
    }

    pub fn get_items(
        &self,
        filter: &str,
        search_term: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DownloadItem>, AppError> {
        let db = self.db.lock().expect("downloads db mutex poisoned");

        let extension_filter = match filter {
            "images" => Some(vec![
                "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico",
            ]),
            "videos" => Some(vec!["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"]),
            "audio" => Some(vec!["mp3", "wav", "flac", "m4a", "ogg", "aac"]),
            "documents" => Some(vec!["pdf", "doc", "docx", "txt", "md", "rtf", "odt"]),
            "archives" => Some(vec!["zip", "tar", "gz", "7z", "rar", "bz2", "xz"]),
            _ => None,
        };

        let mut sql = String::from(
            "SELECT id, path, name, extension, file_type, size_bytes, created_at, accessed_at
             FROM downloads WHERE 1=1",
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(term) = search_term {
            if !term.is_empty() {
                sql.push_str(" AND name LIKE ?");
                params_vec.push(Box::new(format!("%{}%", term)));
            }
        }

        if let Some(exts) = &extension_filter {
            let placeholders: Vec<String> = exts.iter().map(|_| "?".to_string()).collect();
            sql.push_str(&format!(" AND extension IN ({})", placeholders.join(", ")));
            for ext in exts {
                params_vec.push(Box::new(ext.to_string()));
            }
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params_vec.push(Box::new(limit));
        params_vec.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = db.prepare(&sql)?;
        let items_iter = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DownloadItem {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                extension: row.get(3)?,
                file_type: row.get(4)?,
                size_bytes: row.get(5)?,
                created_at: row.get(6)?,
                accessed_at: row.get(7)?,
                is_complete: true,
            })
        })?;

        items_iter
            .collect::<RusqliteResult<Vec<_>>>()
            .map_err(|e| e.into())
    }

    pub fn mark_accessed(&self, id: i64) -> Result<(), AppError> {
        let db = self.db.lock().expect("downloads db mutex poisoned");
        let now = Utc::now().to_rfc3339();
        db.execute(
            "UPDATE downloads SET accessed_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn delete_item(&self, id: i64) -> Result<(), AppError> {
        let db = self.db.lock().expect("downloads db mutex poisoned");
        db.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), AppError> {
        let db = self.db.lock().expect("downloads db mutex poisoned");
        db.execute("DELETE FROM downloads", [])?;
        Ok(())
    }

    /// Scan existing files in a directory and add them to the database
    pub fn scan_directory(&self, dir: &Path) -> Result<usize, AppError> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(0),
        };

        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if self.add_download(&path)?.is_some() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get the downloads directory path
    pub fn get_downloads_dir() -> Option<std::path::PathBuf> {
        dirs::download_dir()
    }
}

// Global manager instance
use once_cell::sync::Lazy;
pub static MANAGER: Lazy<Mutex<Option<DownloadsManager>>> = Lazy::new(|| Mutex::new(None));
