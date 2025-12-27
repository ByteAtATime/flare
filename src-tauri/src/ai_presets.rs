use crate::error::AppError;
use crate::store::{Storable, Store};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use uuid::Uuid;

const AI_PRESETS_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS ai_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    template TEXT NOT NULL,
    icon TEXT,
    created_at INTEGER NOT NULL
)";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiPreset {
    pub id: String,
    pub name: String,
    pub template: String,
    pub icon: Option<String>,
    pub created_at: i64,
}

impl Storable for AiPreset {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(AiPreset {
            id: row.get(0)?,
            name: row.get(1)?,
            template: row.get(2)?,
            icon: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

pub struct AiPresetManager {
    store: Arc<Store>,
}

impl AiPresetManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let store = Arc::new(Store::new(app_handle, "ai_presets.sqlite")?);
        store.init_table(AI_PRESETS_SCHEMA)?;

        // Seed default presets if empty
        let count: i64 = store.conn().query_row(
            "SELECT COUNT(*) FROM ai_presets",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let defaults = vec![
                (
                    "Summarize Selection",
                    "Summarize the following text concisely:\n\n{selection}",
                    "text-align-left",
                ),
                (
                    "Fix Spelling & Grammar",
                    "Fix the spelling and grammar in the following text. Output only the corrected text:\n\n{selection}",
                    "pencil-1",
                ),
                (
                    "Explain Code",
                    "Explain the following code snippet:\n\n{selection}",
                    "code",
                ),
                (
                    "Improve Writing",
                    "Rewrite the following text to be more clear and professional:\n\n{selection}",
                    "magic-wand",
                ),
            ];

            for (name, template, icon) in defaults {
                let id = Uuid::new_v4().to_string();
                let now = chrono::Utc::now().timestamp();
                store.execute(
                    "INSERT INTO ai_presets (id, name, template, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, name, template, icon, now],
                )?;
            }
        }

        Ok(Self { store })
    }

    pub fn get_all(&self) -> Result<Vec<AiPreset>, AppError> {
        self.store.query("SELECT id, name, template, icon, created_at FROM ai_presets ORDER BY name ASC", [])
    }

    pub fn create(&self, name: String, template: String, icon: Option<String>) -> Result<AiPreset, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        self.store.execute(
            "INSERT INTO ai_presets (id, name, template, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, template, icon, now],
        )?;

        Ok(AiPreset {
            id,
            name,
            template,
            icon,
            created_at: now,
        })
    }

    pub fn update(&self, id: String, name: String, template: String, icon: Option<String>) -> Result<(), AppError> {
        self.store.execute(
            "UPDATE ai_presets SET name = ?1, template = ?2, icon = ?3 WHERE id = ?4",
            params![name, template, icon, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: String) -> Result<(), AppError> {
        self.store.execute("DELETE FROM ai_presets WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[tauri::command]
pub fn get_ai_presets(manager: State<AiPresetManager>) -> Result<Vec<AiPreset>, String> {
    manager.get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_ai_preset(
    manager: State<AiPresetManager>,
    name: String,
    template: String,
    icon: Option<String>,
) -> Result<AiPreset, String> {
    manager.create(name, template, icon).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_ai_preset(
    manager: State<AiPresetManager>,
    id: String,
    name: String,
    template: String,
    icon: Option<String>,
) -> Result<(), String> {
    manager.update(id, name, template, icon).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_ai_preset(manager: State<AiPresetManager>, id: String) -> Result<(), String> {
    manager.delete(id).map_err(|e| e.to_string())
}
