use crate::error::AppError;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

const NOTES_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS floating_notes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    updated_at INTEGER NOT NULL
)";

pub struct FloatingNotesManager {
    store: Arc<Mutex<rusqlite::Connection>>,
}

impl FloatingNotesManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let app_dir = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|_| AppError::DirectoryNotFound)?;

        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }

        let db_path = app_dir.join("floating_notes.db");
        let store = rusqlite::Connection::open(db_path)?;

        store.execute(NOTES_SCHEMA, [])?;

        Ok(Self {
            store: Arc::new(Mutex::new(store)),
        })
    }

    pub fn get_content(&self) -> Result<String, AppError> {
        let store = self.store.lock().expect("notes store mutex poisoned");
        let mut stmt = store.prepare("SELECT content FROM floating_notes WHERE id = 'main'")?;

        let result = stmt.query_row([], |row| row.get::<_, String>(0));

        match result {
            Ok(content) => Ok(content),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn save_content(&self, content: String) -> Result<(), AppError> {
        let store = self.store.lock().expect("notes store mutex poisoned");
        let now = chrono::Utc::now().timestamp();

        store.execute(
            "INSERT OR REPLACE INTO floating_notes (id, content, updated_at) VALUES ('main', ?1, ?2)",
            params![content, now],
        )?;
        Ok(())
    }
}

#[tauri::command]
pub fn get_floating_note(manager: State<FloatingNotesManager>) -> Result<String, String> {
    manager.get_content().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_floating_note(
    manager: State<FloatingNotesManager>,
    content: String,
) -> Result<(), String> {
    manager.save_content(content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_floating_notes_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating-notes") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        // Create the window
        let window = WebviewWindowBuilder::new(
            &app,
            "floating-notes",
            WebviewUrl::App("/floating-notes".into()),
        )
        .title("Floating Notes")
        .inner_size(300.0, 400.0)
        .min_inner_size(200.0, 200.0)
        .always_on_top(true)
        .decorations(false) // Frameless
        .transparent(true)
        .build()
        .map_err(|e| e.to_string())?;

        // window.set_position(...) could be restored here if we stored it
    }
    Ok(())
}
