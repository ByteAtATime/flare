use super::{manager::FileSearchManager, types::IndexedFile};
use std::{env, path::PathBuf, time::SystemTime};
use tauri::{AppHandle, Manager};
use walkdir::{DirEntry, WalkDir};

pub async fn build_initial_index(app_handle: AppHandle) {
    tracing::info!("Starting initial file index build");
    let manager = app_handle.state::<FileSearchManager>();
    let home_dir = match env::var("HOME") {
        Ok(path) => path,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get home directory");
            return;
        }
    };

    // Index only specific directories, not entire home
    let index_dirs = [
        "Documents",
        "Downloads",
        "Desktop",
        "Pictures",
        "Videos",
        "Music",
        "Projects",
        "Code",
        "dev",
        "workspace",
    ];

    // Load all existing file timestamps in a single query to avoid N+1 problem
    let existing_files = match manager.get_all_file_timestamps() {
        Ok(timestamps) => timestamps,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load existing file timestamps");
            std::collections::HashMap::new()
        }
    };

    let mut total_indexed = 0;
    for dir_name in &index_dirs {
        let dir_path = PathBuf::from(&home_dir).join(dir_name);
        if !dir_path.exists() || !dir_path.is_dir() {
            continue;
        }

        tracing::info!(path = %dir_path.display(), "Indexing directory");

        // Collect files to add in batches for better performance
        let mut files_to_add = Vec::new();

        let walker = WalkDir::new(&dir_path).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e) && !is_excluded(e)) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!(error = %e, "Error walking directory");
                    continue;
                }
            };

            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            let last_modified_secs = metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            // Use in-memory HashMap lookup instead of database query
            if let Some(&indexed_time) = existing_files.get(&path.to_string_lossy().to_string()) {
                if indexed_time >= last_modified_secs {
                    if path.is_dir() {
                        // continue to walk children
                    } else {
                        // skip this file
                        continue;
                    }
                }
            }

            let file_type = if metadata.is_dir() {
                "directory".to_string()
            } else if metadata.is_file() {
                "file".to_string()
            } else {
                continue;
            };

            let indexed_file = IndexedFile {
                path: path.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                parent_path: path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                file_type,
                last_modified: last_modified_secs,
            };

            files_to_add.push(indexed_file);

            // Batch insert every 1000 files to avoid holding too much memory
            if files_to_add.len() >= 1000 {
                if let Err(e) = manager.batch_add_files(&files_to_add) {
                    tracing::error!(error = ?e, "Failed to batch add files");
                } else {
                    total_indexed += files_to_add.len();
                }
                files_to_add.clear();
            }
        }

        // Insert any remaining files
        if !files_to_add.is_empty() {
            if let Err(e) = manager.batch_add_files(&files_to_add) {
                tracing::error!(error = ?e, "Failed to batch add remaining files");
            } else {
                total_indexed += files_to_add.len();
            }
        }
    }

    tracing::info!(count = total_indexed, "Finished initial file index build");
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_excluded(entry: &DirEntry) -> bool {
    let path = entry.path();
    let excluded_dirs = [
        "node_modules",
        ".git",
        ".svn",
        "target",
        "build",
        ".vscode",
        ".idea",
        "__pycache__",
        ".pytest_cache",
        ".mypy_cache",
        ".cache",
        ".local/share/Trash",
        ".gradle",
        ".wine",
        ".wine-qoder",
        ".npm",
        ".cargo",
        ".rustup",
        ".pnpm-store",
        "venv",
        ".venv",
        "Library",
        "Application Support",
        "AppData",
    ];
    path.components().any(|component| {
        if let Some(name) = component.as_os_str().to_str() {
            excluded_dirs
                .iter()
                .any(|&excluded| name == excluded || name.starts_with(&format!("{}.", excluded)))
        } else {
            false
        }
    })
}
