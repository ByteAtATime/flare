pub mod manager;
pub mod types;
pub mod watcher;

use manager::{DownloadsManager, MANAGER};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use types::DownloadItem;

/// Initialize the downloads module
pub fn init(app_handle: AppHandle) {
    // Create the manager
    let downloads_manager = match DownloadsManager::new(&app_handle) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to create DownloadsManager");
            return;
        }
    };

    // Initialize the database
    if let Err(e) = downloads_manager.init_db() {
        tracing::error!(error = ?e, "Failed to initialize downloads database");
        return;
    }

    // Scan existing downloads on first run
    if let Some(downloads_dir) = DownloadsManager::get_downloads_dir() {
        match downloads_manager.scan_directory(&downloads_dir) {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(count, "Indexed existing downloads");
                }
            }
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to scan existing downloads");
            }
        }
    }

    // Store the manager globally
    *MANAGER.lock().expect("downloads manager mutex poisoned") = Some(downloads_manager);

    // Start the file watcher
    let watcher_handle = app_handle.clone();
    std::thread::spawn(move || {
        if let Err(e) = watcher::start_watching(watcher_handle) {
            tracing::error!(error = %e, "Failed to start downloads watcher");
        }
    });

    tracing::info!("Downloads manager initialized");
}

// Tauri Commands

#[tauri::command]
pub fn downloads_get_items(
    filter: String,
    search_term: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<DownloadItem>, String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        manager
            .get_items(&filter, search_term.as_deref(), limit, offset)
            .map_err(|e| e.to_string())
    } else {
        Err("Downloads manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn downloads_open_file(path: String) -> Result<(), String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    // Mark as accessed
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        // Find the item by path and mark it accessed
        if let Ok(items) = manager.get_items("all", None, 1000, 0) {
            if let Some(item) = items.iter().find(|i| i.path == path.to_string_lossy()) {
                let _ = manager.mark_accessed(item.id);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn downloads_show_in_folder(path: String) -> Result<(), String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    let parent = path.parent().unwrap_or(path);

    #[cfg(target_os = "linux")]
    {
        // Try to use the file manager to highlight the file
        // First try with dbus/nautilus, fall back to xdg-open on parent
        let result = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:file://{}", path.to_string_lossy()),
                "string:",
            ])
            .output();

        if result.is_err() || !result.unwrap().status.success() {
            // Fall back to just opening the folder
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open folder: {}", e))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to show in Finder: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to show in Explorer: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn downloads_delete_item(id: i64) -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        manager.delete_item(id).map_err(|e| e.to_string())
    } else {
        Err("Downloads manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn downloads_delete_file(id: i64, path: String) -> Result<(), String> {
    let path = Path::new(&path);

    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }
    }

    // Also remove from history
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        manager.delete_item(id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn downloads_clear_history() -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        manager.clear_all().map_err(|e| e.to_string())
    } else {
        Err("Downloads manager not initialized".to_string())
    }
}

/// Get the most recent download
#[tauri::command]
pub fn downloads_get_latest() -> Result<Option<DownloadItem>, String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        // Get the first item sorted by created_at descending
        manager
            .get_items("all", None, 1, 0)
            .map(|items| items.into_iter().next())
            .map_err(|e| e.to_string())
    } else {
        Err("Downloads manager not initialized".to_string())
    }
}

/// Copy the latest download path to clipboard
#[tauri::command]
pub fn downloads_copy_latest() -> Result<String, String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("downloads manager mutex poisoned")
        .as_ref()
    {
        match manager.get_items("all", None, 1, 0) {
            Ok(items) => {
                if let Some(item) = items.first() {
                    Ok(item.path.clone())
                } else {
                    Err("No downloads found".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("Downloads manager not initialized".to_string())
    }
}
