use crate::error::AppError;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tracing::{error, info};

/// Application-wide settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // Appearance
    pub theme: String, // "light", "dark", "tokyo-night", "system"
    pub window_opacity: f32,
    pub font_size: String, // "small", "medium", "large"

    // Search Settings
    pub enable_search_history: bool,
    pub search_results_limit: i32,
    pub fuzzy_search_sensitivity: String, // "low", "medium", "high"

    // Window Behavior
    pub close_on_blur: bool,
    pub remember_window_position: bool,
    pub default_window_width: i32,
    pub default_window_height: i32,

    // Developer Options
    pub developer_mode: bool,
    pub show_extension_console: bool,
    pub debug_log_level: String, // "error", "warn", "info", "debug", "trace"

    // Performance
    pub max_concurrent_extensions: i32,
    pub cache_size_mb: i32,
    pub indexing_throttle_ms: i32,

    // System Integration
    pub auto_start_on_login: bool,
    pub clipboard_history_retention_days: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Appearance
            theme: "system".to_string(),
            window_opacity: 1.0,
            font_size: "medium".to_string(),

            // Search Settings
            enable_search_history: true,
            search_results_limit: 50,
            fuzzy_search_sensitivity: "medium".to_string(),

            // Window Behavior
            close_on_blur: false,
            remember_window_position: true,
            default_window_width: 800,
            default_window_height: 600,

            // Developer Options
            developer_mode: false,
            show_extension_console: false,
            debug_log_level: "info".to_string(),

            // Performance
            max_concurrent_extensions: 5,
            cache_size_mb: 100,
            indexing_throttle_ms: 500,

            // System Integration
            auto_start_on_login: false,
            clipboard_history_retention_days: 30,
        }
    }
}

pub struct SettingsManager {
    store: Arc<Store>,
}

impl SettingsManager {
    const SETTINGS_KEY: &'static str = "app_settings";

    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let store = Arc::new(Store::new(app_handle, "flareup.db")?);

        // Initialize settings table
        store.init_table(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )?;

        Ok(Self { store })
    }

    /// Get current application settings, returning defaults if not found
    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let result = self.store.query_row::<SettingsRow, _>(
            "SELECT value FROM app_settings WHERE key = ?1",
            [Self::SETTINGS_KEY],
        )?;

        match result {
            Some(row) => {
                let settings: AppSettings = serde_json::from_str(&row.value).map_err(|e| {
                    error!("Failed to deserialize settings: {}", e);
                    AppError::Serialization(format!("Invalid settings format: {}", e))
                })?;
                info!("Loaded application settings from database");
                Ok(settings)
            }
            None => {
                info!("No settings found in database, using defaults");
                Ok(AppSettings::default())
            }
        }
    }

    /// Save application settings to database
    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        let value = serde_json::to_string(settings).map_err(|e| {
            error!("Failed to serialize settings: {}", e);
            AppError::Serialization(format!("Failed to serialize settings: {}", e))
        })?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::Serialization(format!("Time error: {}", e)))?
            .as_secs() as i64;

        self.store.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            (Self::SETTINGS_KEY, value, timestamp),
        )?;

        info!("Saved application settings to database");
        Ok(())
    }

    /// Reset settings to defaults
    pub fn reset_to_defaults(&self) -> Result<AppSettings, AppError> {
        let defaults = AppSettings::default();
        self.save_settings(&defaults)?;
        info!("Reset application settings to defaults");
        Ok(defaults)
    }
}

// Helper struct for database rows
struct SettingsRow {
    value: String,
}

impl crate::store::Storable for SettingsRow {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self { value: row.get(0)? })
    }
}

// Tauri commands
#[tauri::command]
pub fn get_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    let manager = app_handle.state::<SettingsManager>().inner();

    manager.get_settings().map_err(|e| {
        error!("Error getting settings: {}", e);
        format!("Failed to get settings: {}", e)
    })
}

#[tauri::command]
pub fn save_app_settings(app_handle: AppHandle, settings: AppSettings) -> Result<(), String> {
    let manager = app_handle.state::<SettingsManager>().inner();

    manager.save_settings(&settings).map_err(|e| {
        error!("Error saving settings: {}", e);
        format!("Failed to save settings: {}", e)
    })
}

#[tauri::command]
pub fn reset_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    let manager = app_handle.state::<SettingsManager>().inner();

    manager.reset_to_defaults().map_err(|e| {
        error!("Error resetting settings: {}", e);
        format!("Failed to reset settings: {}", e)
    })
}
