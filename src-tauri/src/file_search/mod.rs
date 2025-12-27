pub mod indexer;
pub mod manager;
pub mod types;
pub mod watcher;

use manager::FileSearchManager;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn search_files(
    term: String,
    manager: State<FileSearchManager>,
) -> Result<Vec<types::IndexedFile>, String> {
    manager.search_files(&term, 100).map_err(|e| e.to_string())
}

pub fn init(app_handle: AppHandle) {
    let file_search_manager = match FileSearchManager::new(app_handle.clone()) {
        Ok(manager) => manager,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to create FileSearchManager");
            return;
        }
    };

    if let Err(e) = file_search_manager.init_db() {
        tracing::error!(error = ?e, "Failed to initialize file search database");
        return;
    }

    app_handle.manage(file_search_manager);

    let indexer_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        indexer::build_initial_index(indexer_handle).await;
    });

    let watcher_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = watcher::start_watching(watcher_handle).await {
            tracing::error!(error = ?e, "Failed to start file watcher");
        }
    });
}
