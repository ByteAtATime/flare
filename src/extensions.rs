use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaycastManifest {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub icon: String,
    pub author: Option<String>,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub commands: Vec<Command>,

    pub owner: Option<String>,
    pub access: Option<AccessLevel>,
    pub contributors: Option<Vec<String>>,
    pub past_contributors: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub preferences: Option<Vec<Preference>>,
    pub external: Option<Vec<String>>,

    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub name: String,
    pub title: String,
    pub description: String,
    pub mode: CommandMode,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub interval: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub arguments: Option<Vec<Argument>>,
    pub preferences: Option<Vec<Preference>>,
    pub disabled_by_default: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandMode {
    View,
    NoView,
    MenuBar,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub name: String,
    pub description: String,

    // TODO: documentation states title is required, but it appears many extensions omit it
    // I believe the idea is that only certain preferences need titles?
    // for now, I'll leave it optional just in case
    pub title: Option<String>,

    #[serde(rename = "type")]
    pub preference_type: PreferenceType,

    pub required: bool,
    pub placeholder: Option<String>,
    pub default: Option<Value>,
    pub label: Option<String>,
    pub data: Option<Vec<DropdownData>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PreferenceType {
    Textfield,
    Password,
    Checkbox,
    Dropdown,
    AppPicker,
    File,
    Directory,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    pub name: String,

    #[serde(rename = "type")]
    pub argument_type: ArgumentType,

    pub placeholder: String,
    pub required: Option<bool>,
    pub data: Option<Vec<DropdownData>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ArgumentType {
    Text,
    Password,
    Dropdown,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DropdownData {
    pub title: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Platform {
    #[serde(rename = "macOS")]
    MacOs,
    #[serde(rename = "Windows")]
    Windows,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AccessLevel {
    Public,
    Private,
}

pub fn get_extensions_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("flare").join("extensions"))
}

pub fn scan_extensions() {
    let Some(extensions_dir) = get_extensions_dir() else {
        eprintln!("Couldn't find a valid path for extensions");
        return;
    };

    if !extensions_dir.exists() {
        eprintln!("Extensions directory does not exist: {:?}", extensions_dir);
        return;
    }

    match fs::read_dir(&extensions_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let package_json = path.join("package.json");
                        if package_json.exists() {
                            match fs::read_to_string(&package_json) {
                                Ok(content) => {
                                    match serde_json::from_str::<RaycastManifest>(&content) {
                                        Ok(manifest) => {
                                            println!(
                                                "ooo extension: {} ({})",
                                                manifest.title, manifest.name
                                            );
                                            for cmd in manifest.commands {
                                                println!(
                                                    "  command: {} ({}), mode: {:?}",
                                                    cmd.title, cmd.name, cmd.mode
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error parsing package.json for {:?}: {}",
                                                path.file_name().unwrap_or_default(),
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Error reading package.json for {:?}: {}",
                                        path.file_name().unwrap_or_default(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading extensions: {}", e);
        }
    }
}
