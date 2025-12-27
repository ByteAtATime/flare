use super::{manager::FileSearchManager, types::IndexedFile};
use crate::error::AppError;
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, DebouncedEvent};
use std::{
    env,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tauri::{AppHandle, Manager};

/// Directories to exclude from file watching
const EXCLUDED_DIRS: &[&str] = &[
    ".wine",
    ".wine-qoder",
    ".cache",
    ".local/share/Trash",
    ".gradle",
    "node_modules",
    ".npm",
    ".cargo",
    ".rustup",
    ".pnpm-store",
    "target",
    "build",
    ".git",
    ".svn",
    ".venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "venv",
];

/// Check if a path should be excluded from watching
fn should_exclude_path(path: &Path) -> bool {
    path.components().any(|component| {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(name) = os_str.to_str() {
                return EXCLUDED_DIRS.iter().any(|excluded| {
                    name == *excluded || name.starts_with(&format!("{}.", excluded))
                });
            }
        }
        false
    })
}

async fn handle_event(app_handle: AppHandle, debounced_event: DebouncedEvent) {
    let manager = app_handle.state::<FileSearchManager>();
    let path = &debounced_event.event.paths[0];

    // Skip excluded paths
    if should_exclude_path(path) {
        return;
    }

    if path.exists() {
        if let Ok(metadata) = path.metadata() {
            let file_type = if metadata.is_dir() {
                "directory".to_string()
            } else {
                "file".to_string()
            };
            let last_modified = metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let indexed_file = IndexedFile {
                path: path.to_string_lossy().to_string(),
                name: path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                parent_path: path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                file_type,
                last_modified,
            };
            if let Err(e) = manager.add_file(&indexed_file) {
                tracing::error!(
                    error = ?e,
                    path = %path.display(),
                    "Failed to add/update file in index"
                );
            }
        }
    } else if let Err(e) = manager.remove_file(&path.to_string_lossy()) {
        tracing::error!(
            error = ?e,
            path = %path.display(),
            "Failed to remove file from index"
        );
    }
}

pub async fn start_watching(app_handle: AppHandle) -> Result<(), AppError> {
    let home_dir = env::var("HOME").map_err(|e| AppError::FileSearch(e.to_string()))?;
    let app_handle_clone = app_handle.clone();

    let mut debouncer = new_debouncer(
        Duration::from_secs(2),
        None,
        move |result: DebounceEventResult| {
            let app_handle_clone2 = app_handle_clone.clone();
            match result {
                Ok(events) => {
                    for event in events {
                        tauri::async_runtime::spawn(handle_event(app_handle_clone2.clone(), event));
                    }
                }
                Err(errors) => {
                    for error in errors {
                        tracing::error!(error = ?error, "File watch error");
                    }
                }
            }
        },
    )
    .map_err(|e| AppError::FileSearch(e.to_string()))?;

    // Watch only specific common directories instead of entire home
    let watch_dirs = [
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

    let mut watch_count = 0;
    for dir_name in &watch_dirs {
        let dir_path = PathBuf::from(&home_dir).join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            if let Err(e) = debouncer
                .watcher()
                .watch(&dir_path, RecursiveMode::Recursive)
            {
                tracing::error!(error = ?e, path = %dir_path.display(), "Failed to watch directory");
            } else {
                debouncer
                    .cache()
                    .add_root(&dir_path, RecursiveMode::Recursive);
                watch_count += 1;
            }
        }
    }

    if watch_count == 0 {
        tracing::warn!("No directories are being watched for file search");
    } else {
        tracing::info!(count = watch_count, "Watching directories for file changes");
    }

    app_handle.manage(debouncer);

    Ok(())
}
