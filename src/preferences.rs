use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::extensions::Extension;

pub type ExtensionPreferences = HashMap<String, Value>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferenceStore {
    extensions: HashMap<String, ExtensionPreferences>,
}

fn get_preferences_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("flare").join("preferences.json");
    }
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".flare").join("preferences.json");
    }
    PathBuf::from(".flare").join("preferences.json")
}

impl PreferenceStore {
    pub fn load() -> Self {
        let path = get_preferences_path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_preferences_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn get_value(
        &self,
        extension_id: &str,
        key: &str,
        extensions: &[Extension],
    ) -> Option<Value> {
        if let Some(ext_prefs) = self.extensions.get(extension_id) {
            if let Some(value) = ext_prefs.get(key) {
                return Some(value.clone());
            }
        }

        extensions
            .iter()
            .find(|ext| ext.manifest.name == extension_id)
            .and_then(|ext| {
                ext.manifest
                    .preferences
                    .as_ref()?
                    .iter()
                    .find(|p| p.name == key)?
                    .default
                    .clone()
            })
    }

    pub fn set_value(&mut self, extension_id: &str, key: &str, value: Value) {
        self.extensions
            .entry(extension_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    pub fn get_extension_preferences(
        &self,
        extension_id: &str,
        extensions: &[Extension],
    ) -> ExtensionPreferences {
        let ext = extensions.iter().find(|e| e.manifest.name == extension_id);

        let mut prefs = ExtensionPreferences::new();

        if let Some(ext) = ext {
            if let Some(manifest_prefs) = &ext.manifest.preferences {
                for pref in manifest_prefs {
                    if let Some(default) = &pref.default {
                        prefs.insert(pref.name.clone(), default.clone());
                    }
                }
            }
        }

        if let Some(saved_prefs) = self.extensions.get(extension_id) {
            for (key, value) in saved_prefs {
                prefs.insert(key.clone(), value.clone());
            }
        }

        prefs
    }
}
