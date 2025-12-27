use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Hotkey configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub command_id: String,
    pub hotkey: String, // Display format: "Ctrl+Alt+←"
    pub modifiers: u8,  // Bitmask: 1=Ctrl, 2=Alt, 4=Shift, 8=Super
    pub key: String,    // Key code: "ArrowLeft", "KeyV", etc.
}

/// Hotkey manager handles registration and persistence
pub struct HotkeyManager {
    store: Arc<Mutex<Connection>>,
    registered: Arc<Mutex<HashMap<String, Shortcut>>>,
}

impl HotkeyManager {
    /// Create new hotkey manager and initialize database
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;

        let db_path = app_dir.join("hotkeys.db");
        let store = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open hotkeys database: {}", e))?;

        // Create table if not exists
        store
            .execute(
                "CREATE TABLE IF NOT EXISTS hotkeys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command_id TEXT NOT NULL UNIQUE,
                hotkey TEXT NOT NULL,
                modifiers INTEGER NOT NULL,
                key TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
                params![],
            )
            .map_err(|e| format!("Failed to create hotkeys table: {}", e))?;

        store
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_hotkeys_command ON hotkeys(command_id)",
                params![],
            )
            .map_err(|e| e.to_string())?;

        store
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_hotkeys_lookup ON hotkeys(modifiers, key)",
                params![],
            )
            .map_err(|e| e.to_string())?;

        // Auto-initialize defaults on first run
        let count: i64 = store
            .query_row("SELECT COUNT(*) FROM hotkeys", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count hotkeys: {}", e))?;

        if count == 0 {
            tracing::info!("First run detected, initializing default hotkeys");
            let defaults = get_default_hotkeys();
            for config in defaults {
                store
                    .execute(
                        "INSERT INTO hotkeys (command_id, hotkey, modifiers, key)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![
                            &config.command_id,
                            &config.hotkey,
                            config.modifiers,
                            &config.key
                        ],
                    )
                    .map_err(|e| format!("Failed to insert default hotkey: {}", e))?;
            }
            tracing::info!(
                "Initialized {} default hotkeys",
                get_default_hotkeys().len()
            );
        } else {
            tracing::info!("Found {} existing hotkeys", count);
        }

        tracing::info!("Hotkey manager initialized");

        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            registered: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Load all hotkeys from database
    pub fn get_all_hotkeys(&self) -> Result<Vec<HotkeyConfig>, String> {
        let store = self.store.lock().expect("hotkey store mutex poisoned");

        let mut stmt = store
            .prepare("SELECT command_id, hotkey, modifiers, key FROM hotkeys ORDER BY command_id")
            .map_err(|e| e.to_string())?;

        let hotkeys = stmt
            .query_map(params![], |row| {
                Ok(HotkeyConfig {
                    command_id: row.get(0)?,
                    hotkey: row.get(1)?,
                    modifiers: row.get(2)?,
                    key: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(hotkeys)
    }

    /// Save a hotkey configuration
    pub fn save_hotkey(&self, config: &HotkeyConfig) -> Result<(), String> {
        let store = self.store.lock().expect("hotkey store mutex poisoned");

        store
            .execute(
                "INSERT OR REPLACE INTO hotkeys (command_id, hotkey, modifiers, key, updated_at)
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
                params![
                    &config.command_id,
                    &config.hotkey,
                    config.modifiers,
                    &config.key
                ],
            )
            .map_err(|e| format!("Failed to save hotkey: {}", e))?;

        tracing::info!("Saved hotkey for {}: {}", config.command_id, config.hotkey);
        Ok(())
    }

    /// Remove a hotkey configuration
    pub fn remove_hotkey(&self, command_id: &str) -> Result<(), String> {
        let store = self.store.lock().expect("hotkey store mutex poisoned");

        store
            .execute(
                "DELETE FROM hotkeys WHERE command_id = ?1",
                params![command_id],
            )
            .map_err(|e| format!("Failed to remove hotkey: {}", e))?;

        tracing::info!("Removed hotkey for {}", command_id);
        Ok(())
    }

    /// Check if a hotkey combination is already in use
    pub fn detect_conflict(&self, modifiers: u8, key: &str) -> Result<Option<String>, String> {
        let store = self.store.lock().expect("hotkey store mutex poisoned");

        let mut stmt = store
            .prepare("SELECT command_id FROM hotkeys WHERE modifiers = ?1 AND key = ?2")
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![modifiers, key], |row| row.get::<_, String>(0));

        match result {
            Ok(command_id) => Ok(Some(command_id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Register a hotkey with Tauri
    pub fn register_shortcut(
        &self,
        app: &AppHandle,
        command_id: String,
        shortcut: Shortcut,
    ) -> Result<(), String> {
        // Register the shortcut
        app.global_shortcut()
            .register(shortcut)
            .map_err(|e| format!("Failed to register hotkey: {}", e))?;

        // Set up the handler
        let command_id_clone = command_id.clone();
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, _, event| {
                if event.state() == ShortcutState::Pressed {
                    tracing::debug!("Hotkey pressed for command: {}", command_id_clone);
                    // Emit event to execute command
                    let _ = app.emit_to(
                        tauri::EventTarget::labeled("main"),
                        "execute-command",
                        &command_id_clone,
                    );
                }
            })
            .map_err(|e| format!("Failed to set hotkey handler: {}", e))?;

        // Track registered shortcut
        let mut registered = self
            .registered
            .lock()
            .expect("registered hotkeys mutex poisoned");
        registered.insert(command_id.clone(), shortcut);

        tracing::info!("Registered hotkey for command: {}", command_id);
        Ok(())
    }

    /// Unregister a hotkey from Tauri
    pub fn unregister_shortcut(&self, app: &AppHandle, command_id: &str) -> Result<(), String> {
        let mut registered = self
            .registered
            .lock()
            .expect("registered hotkeys mutex poisoned");

        if let Some(shortcut) = registered.remove(command_id) {
            app.global_shortcut()
                .unregister(shortcut)
                .map_err(|e| format!("Failed to unregister hotkey: {}", e))?;

            tracing::info!("Unregistered hotkey for command: {}", command_id);
        }

        Ok(())
    }

    /// Get the command ID for a registered shortcut
    #[allow(dead_code)]
    pub fn get_command_for_shortcut(&self, shortcut: &Shortcut) -> Option<String> {
        let registered = self
            .registered
            .lock()
            .expect("registered hotkeys mutex poisoned");
        registered
            .iter()
            .find(|(_, s)| *s == shortcut)
            .map(|(cmd, _)| cmd.clone())
    }
}

/// Convert modifiers bitmask to Tauri Modifiers
pub fn modifiers_from_bits(bits: u8) -> Option<Modifiers> {
    let mut mods = Modifiers::empty();

    if bits & 1 != 0 {
        mods |= Modifiers::CONTROL;
    }
    if bits & 2 != 0 {
        mods |= Modifiers::ALT;
    }
    if bits & 4 != 0 {
        mods |= Modifiers::SHIFT;
    }
    if bits & 8 != 0 {
        mods |= Modifiers::SUPER;
    }

    if mods.is_empty() {
        None
    } else {
        Some(mods)
    }
}

/// Convert Tauri Modifiers to bitmask
#[allow(dead_code)]
pub fn modifiers_to_bits(mods: Modifiers) -> u8 {
    let mut bits = 0u8;

    if mods.contains(Modifiers::CONTROL) {
        bits |= 1;
    }
    if mods.contains(Modifiers::ALT) {
        bits |= 2;
    }
    if mods.contains(Modifiers::SHIFT) {
        bits |= 4;
    }
    if mods.contains(Modifiers::SUPER) {
        bits |= 8;
    }

    bits
}

/// Convert string to Code (key code)
pub fn string_to_code(key: &str) -> Option<Code> {
    match key {
        "ArrowLeft" => Some(Code::ArrowLeft),
        "ArrowRight" => Some(Code::ArrowRight),
        "ArrowUp" => Some(Code::ArrowUp),
        "ArrowDown" => Some(Code::ArrowDown),
        "Space" => Some(Code::Space),
        "Enter" => Some(Code::Enter),
        "Escape" => Some(Code::Escape),
        "Backspace" => Some(Code::Backspace),
        "Tab" => Some(Code::Tab),

        // Letters
        s if s.starts_with("Key") && s.len() == 4 => {
            let letter = s.chars().nth(3)?;
            match letter {
                'A' => Some(Code::KeyA),
                'B' => Some(Code::KeyB),
                'C' => Some(Code::KeyC),
                'D' => Some(Code::KeyD),
                'E' => Some(Code::KeyE),
                'F' => Some(Code::KeyF),
                'G' => Some(Code::KeyG),
                'H' => Some(Code::KeyH),
                'I' => Some(Code::KeyI),
                'J' => Some(Code::KeyJ),
                'K' => Some(Code::KeyK),
                'L' => Some(Code::KeyL),
                'M' => Some(Code::KeyM),
                'N' => Some(Code::KeyN),
                'O' => Some(Code::KeyO),
                'P' => Some(Code::KeyP),
                'Q' => Some(Code::KeyQ),
                'R' => Some(Code::KeyR),
                'S' => Some(Code::KeyS),
                'T' => Some(Code::KeyT),
                'U' => Some(Code::KeyU),
                'V' => Some(Code::KeyV),
                'W' => Some(Code::KeyW),
                'X' => Some(Code::KeyX),
                'Y' => Some(Code::KeyY),
                'Z' => Some(Code::KeyZ),
                _ => None,
            }
        }

        // Numbers
        "Digit0" => Some(Code::Digit0),
        "Digit1" => Some(Code::Digit1),
        "Digit2" => Some(Code::Digit2),
        "Digit3" => Some(Code::Digit3),
        "Digit4" => Some(Code::Digit4),
        "Digit5" => Some(Code::Digit5),
        "Digit6" => Some(Code::Digit6),
        "Digit7" => Some(Code::Digit7),
        "Digit8" => Some(Code::Digit8),
        "Digit9" => Some(Code::Digit9),

        // Symbols
        "Minus" => Some(Code::Minus),
        "Equal" => Some(Code::Equal),
        "BracketLeft" => Some(Code::BracketLeft),
        "BracketRight" => Some(Code::BracketRight),
        "Backslash" => Some(Code::Backslash),
        "Semicolon" => Some(Code::Semicolon),
        "Quote" => Some(Code::Quote),
        "Comma" => Some(Code::Comma),
        "Period" => Some(Code::Period),
        "Slash" => Some(Code::Slash),

        _ => None,
    }
}

/// Format modifiers and key as display string
pub fn format_hotkey(modifiers: u8, key: &str) -> String {
    let mut parts = Vec::new();

    if modifiers & 8 != 0 {
        parts.push("Super");
    }
    if modifiers & 1 != 0 {
        parts.push("Ctrl");
    }
    if modifiers & 2 != 0 {
        parts.push("Alt");
    }
    if modifiers & 4 != 0 {
        parts.push("Shift");
    }

    // Format key
    let key_display = match key {
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        s if s.starts_with("Key") => &s[3..],   // "KeyV" -> "V"
        s if s.starts_with("Digit") => &s[5..], // "Digit5" -> "5"
        s => s,
    };

    parts.push(key_display);
    parts.join("+")
}

/// Get default hotkey configurations
pub fn get_default_hotkeys() -> Vec<HotkeyConfig> {
    vec![
        // System Commands
        HotkeyConfig {
            command_id: "builtin:lock-screen".to_string(),
            hotkey: "Ctrl+Alt+L".to_string(),
            modifiers: 1 | 2,
            key: "KeyL".to_string(),
        },
        // Built-in Features
        HotkeyConfig {
            command_id: "builtin:history".to_string(),
            hotkey: "Ctrl+Shift+V".to_string(),
            modifiers: 1 | 4, // Ctrl + Shift
            key: "KeyV".to_string(),
        },
        HotkeyConfig {
            command_id: "builtin:search-snippets".to_string(),
            hotkey: "Ctrl+Shift+S".to_string(),
            modifiers: 1 | 4,
            key: "KeyS".to_string(),
        },
    ]
}

// Tauri commands

#[tauri::command]
pub async fn get_hotkey_config(app: AppHandle) -> Result<Vec<HotkeyConfig>, String> {
    let manager = app.state::<HotkeyManager>();
    manager.get_all_hotkeys()
}

#[tauri::command]
pub async fn set_command_hotkey(
    app: AppHandle,
    command_id: String,
    modifiers: u8,
    key: String,
) -> Result<(), String> {
    let manager = app.state::<HotkeyManager>();

    // Check for conflicts
    if let Some(conflict) = manager.detect_conflict(modifiers, &key)? {
        if conflict != command_id {
            return Err(format!("Hotkey already assigned to: {}", conflict));
        }
    }

    // Create config
    let hotkey_display = format_hotkey(modifiers, &key);
    let config = HotkeyConfig {
        command_id: command_id.clone(),
        hotkey: hotkey_display,
        modifiers,
        key: key.clone(),
    };

    // Save to database
    manager.save_hotkey(&config)?;

    // Unregister old shortcut if exists
    let _ = manager.unregister_shortcut(&app, &command_id);

    // Register new shortcut
    let mods = modifiers_from_bits(modifiers).ok_or("Invalid modifiers")?;
    let code = string_to_code(&key).ok_or("Invalid key code")?;
    let shortcut = Shortcut::new(Some(mods), code);

    manager.register_shortcut(&app, command_id, shortcut)?;

    Ok(())
}

#[tauri::command]
pub async fn remove_command_hotkey(app: AppHandle, command_id: String) -> Result<(), String> {
    let manager = app.state::<HotkeyManager>();

    // Unregister from Tauri
    manager.unregister_shortcut(&app, &command_id)?;

    // Remove from database
    manager.remove_hotkey(&command_id)?;

    Ok(())
}

#[tauri::command]
pub async fn check_hotkey_conflict(
    app: AppHandle,
    modifiers: u8,
    key: String,
) -> Result<Option<String>, String> {
    let manager = app.state::<HotkeyManager>();
    manager.detect_conflict(modifiers, &key)
}

#[tauri::command]
pub async fn reset_hotkeys_to_defaults(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<HotkeyManager>();

    // Get all current hotkeys and unregister them
    let current = manager.get_all_hotkeys()?;
    for config in current {
        let _ = manager.unregister_shortcut(&app, &config.command_id);
        let _ = manager.remove_hotkey(&config.command_id);
    }

    // Apply defaults
    let defaults = get_default_hotkeys();
    for config in defaults {
        manager.save_hotkey(&config)?;

        let mods = modifiers_from_bits(config.modifiers).ok_or("Invalid modifiers")?;
        let code = string_to_code(&config.key).ok_or("Invalid key code")?;
        let shortcut = Shortcut::new(Some(mods), code);

        let _ = manager.register_shortcut(&app, config.command_id, shortcut);
    }

    Ok(())
}
