use crate::error::AppError;
use rusqlite::params;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

const ALIASES_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS command_aliases (
    alias TEXT PRIMARY KEY,
    command_id TEXT NOT NULL
)";

pub struct AliasManager {
    store: Arc<Mutex<rusqlite::Connection>>,
}

impl AliasManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let app_dir = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|_| AppError::DirectoryNotFound)?;

        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }

        let db_path = app_dir.join("aliases.db");
        let store = rusqlite::Connection::open(db_path)?;

        store.execute(ALIASES_SCHEMA, [])?;

        Ok(Self {
            store: Arc::new(Mutex::new(store)),
        })
    }

    pub fn get_all(&self) -> Result<HashMap<String, String>, AppError> {
        let store = self.store.lock().expect("alias store mutex poisoned");
        let mut stmt = store.prepare("SELECT alias, command_id FROM command_aliases")?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut aliases = HashMap::new();
        for row in rows {
            let (alias, command_id) = row?;
            aliases.insert(alias, command_id);
        }

        Ok(aliases)
    }

    pub fn set_alias(&self, alias: String, command_id: String) -> Result<(), AppError> {
        let store = self.store.lock().expect("alias store mutex poisoned");
        store.execute(
            "INSERT OR REPLACE INTO command_aliases (alias, command_id) VALUES (?1, ?2)",
            params![alias, command_id],
        )?;
        Ok(())
    }

    pub fn remove_alias(&self, alias: String) -> Result<(), AppError> {
        let store = self.store.lock().expect("alias store mutex poisoned");
        store.execute(
            "DELETE FROM command_aliases WHERE alias = ?1",
            params![alias],
        )?;
        Ok(())
    }
}

#[tauri::command]
pub fn get_aliases(manager: State<AliasManager>) -> Result<HashMap<String, String>, String> {
    manager.get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_alias(
    manager: State<AliasManager>,
    alias: String,
    command_id: String,
) -> Result<(), String> {
    manager.set_alias(alias, command_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_alias(manager: State<AliasManager>, alias: String) -> Result<(), String> {
    manager.remove_alias(alias).map_err(|e| e.to_string())
}
