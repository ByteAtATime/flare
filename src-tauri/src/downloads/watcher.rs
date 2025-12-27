use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::AppHandle;

use super::manager::MANAGER;

/// Start watching the Downloads directory for new files
pub fn start_watching(_app_handle: AppHandle) -> Result<(), String> {
    let downloads_dir = match dirs::download_dir() {
        Some(dir) => dir,
        None => {
            tracing::warn!("Could not determine downloads directory");
            return Err("Could not determine downloads directory".to_string());
        }
    };

    if !downloads_dir.exists() {
        tracing::warn!(path = %downloads_dir.display(), "Downloads directory does not exist");
        return Err("Downloads directory does not exist".to_string());
    }

    // Create a channel to receive events
    let (tx, rx) = mpsc::channel();

    // Create the watcher with a debounce of 500ms
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch the downloads directory
    watcher
        .watch(&downloads_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch downloads directory: {}", e))?;

    tracing::info!(path = %downloads_dir.display(), "Watching downloads directory");

    // Spawn a thread to handle events
    std::thread::spawn(move || {
        // Keep watcher alive
        let _watcher = watcher;

        for event in rx {
            handle_event(event);
        }
    });

    Ok(())
}

fn handle_event(event: Event) {
    // Only handle file creation and rename events
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(notify::event::ModifyKind::Name(_)) => {}
        _ => return,
    }

    for path in event.paths {
        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        // Skip hidden files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        tracing::debug!(path = %path.display(), "New download detected");

        // Add to manager - use Ok pattern to handle poisoned mutex gracefully
        if let Ok(guard) = MANAGER.lock() {
            if let Some(manager) = guard.as_ref() {
                match manager.add_download(&path) {
                    Ok(Some(item)) => {
                        tracing::info!(name = %item.name, "Added download to history");
                    }
                    Ok(None) => {
                        // Skipped (incomplete download or error reading file)
                    }
                    Err(e) => {
                        tracing::error!(error = %e, path = %path.display(), "Failed to add download");
                    }
                }
            }
        }
    }
}

/// Get the downloads directory path
#[allow(dead_code)]
pub fn get_downloads_path() -> Option<PathBuf> {
    dirs::download_dir()
}
