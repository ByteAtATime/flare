use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use notify::{Watcher, RecursiveMode, RecommendedWatcher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScriptMode {
    FullOutput,
    Compact,
    Silent,
    Inline,
}

impl Default for ScriptMode {
    fn default() -> Self {
        ScriptMode::Compact
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptArgument {
    pub name: String,
    pub placeholder: Option<String>,
    pub optional: bool,
    pub percent_encoded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptCommand {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub mode: ScriptMode,
    pub schema_version: u32,
    pub package_name: Option<String>,
    pub icon: Option<String>,
    pub authors: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<ScriptArgument>,
    pub needs_confirmation: bool,
    pub refresh_time: Option<String>,
}

pub struct ScriptCommandManager {
    scripts: Arc<Mutex<HashMap<String, ScriptCommand>>>,
    scripts_dir: PathBuf,
    // Keep watcher alive
    _watcher: Option<RecommendedWatcher>,
}

impl ScriptCommandManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_local_data_dir()
            .expect("failed to get app local data dir");
        let scripts_dir = data_dir.join("scripts");

        if !scripts_dir.exists() {
            let _ = fs::create_dir_all(&scripts_dir);
        }

        let manager = Self {
            scripts: Arc::new(Mutex::new(HashMap::new())),
            scripts_dir: scripts_dir.clone(),
            _watcher: None, // Initialized below
        };

        // Scan initially
        manager.scan_directory();

        // Setup watcher
        let scripts_clone = manager.scripts.clone();
        let dir_clone = scripts_dir.clone();

        let watcher_result = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    // Simple re-scan for now
                    Self::scan_directory_static(&dir_clone, &scripts_clone);
                }
            }
        });

        let mut final_manager = manager;
        if let Ok(mut watcher) = watcher_result {
            let _ = watcher.watch(&scripts_dir, RecursiveMode::Recursive);
            final_manager._watcher = Some(watcher);
        } else {
            tracing::error!("Failed to initialize script watcher");
        }

        final_manager
    }

    fn scan_directory(&self) {
        Self::scan_directory_static(&self.scripts_dir, &self.scripts);
    }

    fn scan_directory_static(dir: &Path, scripts_store: &Arc<Mutex<HashMap<String, ScriptCommand>>>) {
        let mut new_scripts = HashMap::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    // Check if executable
                    if let Ok(metadata) = path.metadata() {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            if let Some(script) = Self::parse_script(&path) {
                                new_scripts.insert(script.path.clone(), script);
                            }
                        }
                    }
                }
            }
        }

        let mut store = scripts_store.lock().unwrap();
        *store = new_scripts;
    }

    fn parse_script(path: &Path) -> Option<ScriptCommand> {
        let content = fs::read_to_string(path).ok()?;

        // Basic Raycast metadata parsing
        // We look for comments like:
        // @raycast.schemaVersion 1
        // @raycast.title My Script
        // @raycast.mode compact

        let mut title = None;
        let mut mode = ScriptMode::Compact;
        let mut schema_version = 1;
        let mut package_name = None;
        let mut icon = None;
        let mut authors = None;
        let mut description = None;
        let mut needs_confirmation = false;
        let mut arguments = Vec::new();

        let re_kv = Regex::new(r"@raycast\.([a-zA-Z0-9]+)\s+(.+)").unwrap();
        let re_arg = Regex::new(r"@raycast\.argument(\d+)\s+(.+)").unwrap();

        for line in content.lines() {
            if let Some(caps) = re_kv.captures(line) {
                let key = caps.get(1)?.as_str();
                let value = caps.get(2)?.as_str().trim();

                match key {
                    "schemaVersion" => schema_version = value.parse().unwrap_or(1),
                    "title" => title = Some(value.to_string()),
                    "mode" => mode = match value {
                        "fullOutput" => ScriptMode::FullOutput,
                        "silent" => ScriptMode::Silent,
                        "inline" => ScriptMode::Inline,
                        _ => ScriptMode::Compact,
                    },
                    "packageName" => package_name = Some(value.to_string()),
                    "icon" => icon = Some(value.to_string()),
                    "author" | "authors" => authors = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "needsConfirmation" => needs_confirmation = value == "true",
                    _ => {
                        if key.starts_with("argument") {
                            // Handled by specific regex below, but this block catches others
                        }
                    }
                }
            }

            if let Some(caps) = re_arg.captures(line) {
                // let _index = caps.get(1)?.as_str(); // We just push in order for now
                let json_str = caps.get(2)?.as_str();
                if let Ok(arg_val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let name = arg_val.get("placeholder").and_then(|v| v.as_str()).unwrap_or("Argument").to_string();
                    let optional = arg_val.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
                    let percent_encoded = arg_val.get("percentEncoded").and_then(|v| v.as_bool()).unwrap_or(false);

                    arguments.push(ScriptArgument {
                        name: name.clone(),
                        placeholder: Some(name), // Use name as placeholder for now
                        optional,
                        percent_encoded,
                    });
                }
            }
        }

        // If no title found, it's not a valid raycast script (or we fallback to filename?)
        // Raycast docs say title is required.
        let title = title?;

        Some(ScriptCommand {
            path: path.to_string_lossy().to_string(),
            filename: path.file_name()?.to_string_lossy().to_string(),
            title,
            mode,
            schema_version,
            package_name,
            icon,
            authors,
            description,
            arguments,
            needs_confirmation,
            refresh_time: None,
        })
    }

    pub fn get_scripts(&self) -> Vec<ScriptCommand> {
        let store = self.scripts.lock().unwrap();
        store.values().cloned().collect()
    }
}

#[tauri::command]
pub fn get_script_commands(manager: State<ScriptCommandManager>) -> Vec<ScriptCommand> {
    manager.get_scripts()
}

#[tauri::command]
pub fn run_script_command(command_path: String, args: Vec<String>) -> Result<String, String> {
    let output = Command::new(&command_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[tauri::command]
pub fn open_scripts_folder(app: AppHandle) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Failed to get data dir".to_string())?;
    let scripts_dir = data_dir.join("scripts");

    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir).map_err(|e| e.to_string())?;
    }

    crate::system::show_in_finder(scripts_dir.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}
