use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: RaycastManifest,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionCommand {
    pub extension_name: String,
    pub extension_title: String,
    pub extension_icon: Option<String>,
    pub extension_path: PathBuf,
    pub command_name: String,
    pub command_title: String,
    pub command_subtitle: Option<String>,
    pub command_icon: Option<String>,
    pub command_mode: CommandMode,
}

pub fn get_extensions_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("flare").join("extensions"))
}

pub fn scan_extensions() -> Vec<Extension> {
    let Some(extensions_dir) = get_extensions_dir() else {
        return Vec::new();
    };

    if !extensions_dir.exists() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(&extensions_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let path = entry.path();
            let package_json = path.join("package.json");

            if !package_json.exists() {
                return None;
            }

            let content = fs::read_to_string(&package_json).ok()?;
            let manifest = serde_json::from_str::<RaycastManifest>(&content).ok()?;

            Some(Extension { manifest, path })
        })
        .collect()
}

pub fn get_launchable_commands(extensions: &[Extension]) -> Vec<ExtensionCommand> {
    extensions
        .iter()
        .flat_map(|ext| {
            let extension_icon = if ext.manifest.icon.is_empty() {
                None
            } else {
                Some(ext.manifest.icon.clone())
            };

            ext.manifest.commands.iter().filter_map(move |cmd| {
                if cmd.mode != CommandMode::View {
                    return None;
                }

                Some(ExtensionCommand {
                    extension_name: ext.manifest.name.clone(),
                    extension_title: ext.manifest.title.clone(),
                    extension_icon: extension_icon.clone(),
                    extension_path: ext.path.clone(),
                    command_name: cmd.name.clone(),
                    command_title: cmd.title.clone(),
                    command_subtitle: cmd.subtitle.clone(),
                    command_icon: cmd.icon.clone(),
                    command_mode: cmd.mode.clone(),
                })
            })
        })
        .collect()
}
