use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;
use zip::result::ZipError;
use zip::ZipArchive;

use crate::cli_substitutes;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeuristicViolation {
    pub command_name: String,
    pub command_title: String,
    pub reason: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum InstallResult {
    Success,
    RequiresConfirmation { violations: Vec<HeuristicViolation> },
}

trait IncompatibilityHeuristic {
    fn check(
        &self,
        command_name: &str,
        command_title: &str,
        file_content: &str,
    ) -> Option<HeuristicViolation>;
}

struct AppleScriptHeuristic;
impl IncompatibilityHeuristic for AppleScriptHeuristic {
    fn check(
        &self,
        command_name: &str,
        command_title: &str,
        file_content: &str,
    ) -> Option<HeuristicViolation> {
        if file_content.contains("runAppleScript") {
            Some(HeuristicViolation {
                command_name: command_name.to_string(),
                command_title: command_title.to_string(),
                reason: "Possible usage of AppleScript (runAppleScript)".to_string(),
            })
        } else {
            None
        }
    }
}

struct MacOSPathHeuristic;
impl IncompatibilityHeuristic for MacOSPathHeuristic {
    fn check(
        &self,
        command_name: &str,
        command_title: &str,
        file_content: &str,
    ) -> Option<HeuristicViolation> {
        let macos_paths = ["/Applications/", "/Library/", "/Users/"];
        for path in macos_paths {
            if file_content.contains(path) {
                return Some(HeuristicViolation {
                    command_name: command_name.to_string(),
                    command_title: command_title.to_string(),
                    reason: format!("Potential hardcoded macOS path: '{}'", path),
                });
            }
        }
        None
    }
}

struct MacOSAPIHeuristic;
impl IncompatibilityHeuristic for MacOSAPIHeuristic {
    fn check(
        &self,
        command_name: &str,
        command_title: &str,
        file_content: &str,
    ) -> Option<HeuristicViolation> {
        let macos_apis = [
            ("NSWorkspace", "macOS NSWorkspace API"),
            ("NSApplication", "macOS NSApplication API"),
            ("NSFileManager", "macOS NSFileManager API"),
            ("com.apple.", "macOS-specific bundle identifier"),
            ("tell app \"Finder\"", "macOS Finder AppleScript"),
            ("tell application \"Finder\"", "macOS Finder AppleScript"),
        ];

        for (pattern, description) in macos_apis {
            if file_content.contains(pattern) {
                return Some(HeuristicViolation {
                    command_name: command_name.to_string(),
                    command_title: command_title.to_string(),
                    reason: format!("Uses {}", description),
                });
            }
        }
        None
    }
}

struct ShellCommandHeuristic;
impl IncompatibilityHeuristic for ShellCommandHeuristic {
    fn check(
        &self,
        command_name: &str,
        command_title: &str,
        file_content: &str,
    ) -> Option<HeuristicViolation> {
        let macos_commands = [
            ("osascript", "macOS osascript command"),
            ("open -a", "macOS application launcher"),
            ("mdfind", "macOS Spotlight search"),
            ("mdls", "macOS Spotlight metadata"),
            ("defaults read", "macOS preferences system"),
            ("defaults write", "macOS preferences system"),
        ];

        for (pattern, description) in macos_commands {
            if file_content.contains(pattern) {
                return Some(HeuristicViolation {
                    command_name: command_name.to_string(),
                    command_title: command_title.to_string(),
                    reason: format!("Uses {}", description),
                });
            }
        }
        None
    }
}

/// Magic bytes for detecting Mach-O binaries (macOS executables)
/// - MH_MAGIC (32-bit): 0xFEEDFACE
/// - MH_CIGAM (32-bit, byte-swapped): 0xCEFAEDFE
/// - MH_MAGIC_64 (64-bit): 0xFEEDFACF
/// - MH_CIGAM_64 (64-bit, byte-swapped): 0xCFFAEDFE
/// - FAT_MAGIC (universal binary): 0xCAFEBABE
/// - FAT_CIGAM (universal, byte-swapped): 0xBEBAFECA
const MACH_O_MAGIC_BYTES: &[[u8; 4]] = &[
    [0xFE, 0xED, 0xFA, 0xCE], // MH_MAGIC
    [0xCE, 0xFA, 0xED, 0xFE], // MH_CIGAM
    [0xFE, 0xED, 0xFA, 0xCF], // MH_MAGIC_64
    [0xCF, 0xFA, 0xED, 0xFE], // MH_CIGAM_64
    [0xCA, 0xFE, 0xBA, 0xBE], // FAT_MAGIC (universal binary)
    [0xBE, 0xBA, 0xFE, 0xCA], // FAT_CIGAM
];

/// Check if the first 4 bytes of data indicate a Mach-O binary
fn is_macho_binary(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    let header: [u8; 4] = [data[0], data[1], data[2], data[3]];
    MACH_O_MAGIC_BYTES.contains(&header)
}

fn get_extension_dir(app: &tauri::AppHandle, slug: &str) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Failed to get app local data dir".to_string())?;
    Ok(data_dir.join("plugins").join(slug))
}

async fn download_archive(url: &str) -> Result<bytes::Bytes, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download extension: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download extension: status code {}",
            response.status()
        ));
    }

    response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))
}

fn find_common_prefix(file_names: &[PathBuf]) -> Option<PathBuf> {
    if file_names.len() <= 1 {
        return None;
    }
    file_names
        .get(0)
        .and_then(|p| p.components().next())
        .and_then(|first_component| {
            if file_names
                .iter()
                .all(|path| path.starts_with(first_component))
            {
                Some(PathBuf::from(first_component.as_os_str()))
            } else {
                None
            }
        })
}

#[derive(Clone)]
struct CommandToCheck {
    path_in_archive: String,
    command_name: String,
    command_title: String,
}

fn get_commands_from_package_json(
    archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    prefix: &Option<PathBuf>,
) -> Result<Vec<CommandToCheck>, String> {
    let package_json_path = if let Some(ref p) = prefix {
        p.join("package.json")
    } else {
        PathBuf::from("package.json")
    };

    let mut pkg_file = match archive.by_name(&package_json_path.to_string_lossy()) {
        Ok(file) => file,
        Err(ZipError::FileNotFound) => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };

    let mut pkg_str = String::new();
    pkg_file
        .read_to_string(&mut pkg_str)
        .map_err(|e| e.to_string())?;

    let pkg_json: serde_json::Value =
        serde_json::from_str(&pkg_str).map_err(|_| "Failed to parse package.json".to_string())?;

    let commands = match pkg_json.get("commands").and_then(|c| c.as_array()) {
        Some(cmds) => cmds,
        None => return Ok(vec![]),
    };

    Ok(commands
        .iter()
        .filter_map(|command| {
            let command_name = command.get("name")?.as_str()?;
            let command_title = command
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or(command_name)
                .to_string();

            let src_path = format!("{}.js", command_name);
            let command_file_path_in_archive = if let Some(ref p) = prefix {
                p.join(src_path)
            } else {
                PathBuf::from(src_path)
            };

            Some(CommandToCheck {
                path_in_archive: command_file_path_in_archive.to_string_lossy().into_owned(),
                command_name: command_name.to_string(),
                command_title,
            })
        })
        .collect())
}

/// Result from heuristic checks, including detected Mach-O binaries for substitution
struct HeuristicResult {
    violations: Vec<HeuristicViolation>,
    macho_binaries: Vec<String>,
}

fn run_heuristic_checks(archive_data: &bytes::Bytes) -> Result<HeuristicResult, String> {
    let heuristics: Vec<Box<dyn IncompatibilityHeuristic + Send + Sync>> = vec![
        Box::new(AppleScriptHeuristic),
        Box::new(MacOSPathHeuristic),
        Box::new(MacOSAPIHeuristic),
        Box::new(ShellCommandHeuristic),
    ];

    let mut archive =
        ZipArchive::new(Cursor::new(archive_data.clone())).map_err(|e| e.to_string())?;
    let file_names: Vec<PathBuf> = archive.file_names().map(PathBuf::from).collect();
    let prefix = find_common_prefix(&file_names);

    let mut violations = Vec::new();

    // Check for Mach-O binaries in assets folder
    let mut macho_binaries_found: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let file_path = file.name().to_string();

            // Skip directories and common non-binary files
            if file.is_dir()
                || file_path.ends_with(".js")
                || file_path.ends_with(".json")
                || file_path.ends_with(".md")
                || file_path.ends_with(".txt")
                || file_path.ends_with(".png")
                || file_path.ends_with(".svg")
                || file_path.ends_with(".jpg")
                || file_path.ends_with(".gif")
                || file_path.ends_with(".css")
                || file_path.ends_with(".html")
            {
                continue;
            }

            // Read first 4 bytes to check for Mach-O magic
            let mut header = [0u8; 4];
            if file.read_exact(&mut header).is_ok() && is_macho_binary(&header) {
                // Get just the filename for the warning message
                let binary_name = Path::new(&file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&file_path)
                    .to_string();
                macho_binaries_found.push(binary_name);
            }
        }
    }

    // Add a single violation for all Mach-O binaries found
    if !macho_binaries_found.is_empty() {
        let binary_list = if macho_binaries_found.len() <= 3 {
            macho_binaries_found.join(", ")
        } else {
            format!(
                "{} and {} more",
                macho_binaries_found[..3].join(", "),
                macho_binaries_found.len() - 3
            )
        };
        violations.push(HeuristicViolation {
            command_name: "_extension".to_string(),
            command_title: "Extension Assets".to_string(),
            reason: format!(
                "Contains macOS-only binary files that won't work on Linux: {}",
                binary_list
            ),
        });
    }

    // Re-open archive for command checks (since we consumed it above)
    let mut archive =
        ZipArchive::new(Cursor::new(archive_data.clone())).map_err(|e| e.to_string())?;

    // Check command source files for incompatibility patterns
    let commands_to_check = get_commands_from_package_json(&mut archive, &prefix)?;
    for command_meta in commands_to_check {
        if let Ok(mut command_file) = archive.by_name(&command_meta.path_in_archive) {
            let mut content = String::new();
            if command_file.read_to_string(&mut content).is_ok() {
                for heuristic in &heuristics {
                    if let Some(violation) = heuristic.check(
                        &command_meta.command_name,
                        &command_meta.command_title,
                        &content,
                    ) {
                        violations.push(violation);
                    }
                }
            }
        }
    }
    Ok(HeuristicResult {
        violations,
        macho_binaries: macho_binaries_found,
    })
}

const COMPATIBILITY_FILE_NAME: &str = "compatibility.json";

#[derive(Serialize, Deserialize, Default)]
struct CompatibilityMetadata {
    #[serde(default)]
    warnings: Vec<HeuristicViolation>,
    #[serde(default = "default_compatibility_score")]
    compatibility_score: u8,
}

fn default_compatibility_score() -> u8 {
    100
}

/// Calculate compatibility score (0-100) based on detected violations
/// Higher score = better Linux compatibility
fn calculate_compatibility_score(violations: &[HeuristicViolation]) -> u8 {
    let mut score: i32 = 100;

    for violation in violations {
        // Deduct points based on severity of the issue
        if violation.reason.contains("macOS-only binary") {
            // Mach-O binaries are a major blocker
            score -= 40;
        } else if violation.reason.contains("macOS NSWorkspace API")
            || violation.reason.contains("macOS NSApplication API")
            || violation.reason.contains("macOS NSFileManager API")
            || violation.reason.contains("macOS Finder AppleScript")
        {
            // macOS-specific APIs likely won't work
            score -= 20;
        } else if violation.reason.contains("AppleScript") {
            // AppleScript is shimmed but has limitations
            score -= 15;
        } else if violation.reason.contains("macOS path") {
            // Paths can be translated
            score -= 10;
        } else if violation.reason.contains("osascript")
            || violation.reason.contains("mdfind")
            || violation.reason.contains("mdls")
            || violation.reason.contains("defaults")
            || violation.reason.contains("open -a")
        {
            // Shell commands are platform-specific
            score -= 5;
        }
    }

    // Clamp to 0-100 range
    score.max(0).min(100) as u8
}

fn save_compatibility_metadata(
    plugin_dir: &Path,
    warnings: &[HeuristicViolation],
) -> Result<(), String> {
    let compatibility_score = calculate_compatibility_score(warnings);
    let metadata = CompatibilityMetadata {
        warnings: warnings.to_vec(),
        compatibility_score,
    };
    let data = serde_json::to_string_pretty(&metadata).map_err(|e| e.to_string())?;
    fs::write(plugin_dir.join(COMPATIBILITY_FILE_NAME), data).map_err(|e| e.to_string())
}

fn load_compatibility_metadata(plugin_dir: &Path) -> Result<CompatibilityMetadata, String> {
    let path = plugin_dir.join(COMPATIBILITY_FILE_NAME);
    if !path.exists() {
        return Ok(CompatibilityMetadata::default());
    }

    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: CompatibilityMetadata = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(parsed)
}

fn extract_archive(archive_data: &bytes::Bytes, target_dir: &Path) -> Result<(), String> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

    let mut archive =
        ZipArchive::new(Cursor::new(archive_data.clone())).map_err(|e| e.to_string())?;
    let file_names: Vec<PathBuf> = archive.file_names().map(PathBuf::from).collect();
    let prefix_to_strip = find_common_prefix(&file_names);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed_path = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };

        let final_path_part = if let Some(ref prefix) = prefix_to_strip {
            enclosed_path
                .strip_prefix(prefix)
                .unwrap_or(&enclosed_path)
                .to_path_buf()
        } else {
            enclosed_path
        };

        if final_path_part.as_os_str().is_empty() {
            continue;
        }

        let outpath = target_dir.join(final_path_part);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Author {
    Simple(String),
    Detailed { name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceData {
    pub title: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub required: Option<bool>,
    #[serde(default)]
    pub default: serde_json::Value,
    pub data: Option<Vec<PreferenceData>>,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct CommandInfo {
    name: String,
    title: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    subtitle: Option<String>,
    mode: Option<String>,
    preferences: Option<Vec<Preference>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PackageJson {
    name: Option<String>,
    title: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    author: Option<Author>,
    owner: Option<String>,
    commands: Option<Vec<CommandInfo>>,
    preferences: Option<Vec<Preference>>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub title: String,
    pub description: Option<String>,
    pub plugin_title: String,
    pub plugin_name: String,
    pub command_name: String,
    pub plugin_path: String,
    pub icon: Option<String>,
    pub preferences: Option<Vec<Preference>>,
    pub command_preferences: Option<Vec<Preference>>,
    pub mode: Option<String>,
    pub author: Option<Author>,
    pub owner: Option<String>,
    pub compatibility_warnings: Option<Vec<HeuristicViolation>>,
    pub compatibility_score: Option<u8>,
}

pub fn discover_plugins(app: &tauri::AppHandle) -> Result<Vec<PluginInfo>, String> {
    let plugins_base_dir = get_extension_dir(app, "")?;
    let mut plugins = Vec::new();

    if !plugins_base_dir.exists() {
        fs::create_dir_all(&plugins_base_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        return Ok(plugins);
    }

    let plugin_dirs = fs::read_dir(plugins_base_dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir());

    for plugin_dir_entry in plugin_dirs {
        let plugin_dir = plugin_dir_entry.path();
        let plugin_dir_name = plugin_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let package_json_path = plugin_dir.join("package.json");

        if !package_json_path.exists() {
            tracing::warn!(plugin = %plugin_dir_name, "Plugin has no package.json, skipping");
            continue;
        }

        let package_json_content = match fs::read_to_string(&package_json_path) {
            Ok(content) => content,
            Err(e) => {
                tracing::warn!(
                    plugin = %plugin_dir_name,
                    error = %e,
                    "Error reading package.json for plugin"
                );
                continue;
            }
        };

        let package_json: PackageJson = match serde_json::from_str(&package_json_content) {
            Ok(json) => json,
            Err(e) => {
                tracing::warn!(
                    plugin = %plugin_dir_name,
                    error = %e,
                    "Error parsing package.json for plugin"
                );
                continue;
            }
        };

        let compatibility_metadata = match load_compatibility_metadata(&plugin_dir) {
            Ok(data) => data,
            Err(err) => {
                tracing::warn!(
                    plugin = %plugin_dir_name,
                    error = %err,
                    "Failed to load compatibility metadata"
                );
                CompatibilityMetadata::default()
            }
        };

        if let Some(commands) = package_json.commands {
            for command in commands {
                let command_file_path = plugin_dir.join(format!("{}.js", command.name));
                if command_file_path.exists() {
                    let warnings: Vec<HeuristicViolation> = compatibility_metadata
                        .warnings
                        .iter()
                        .filter(|warning| warning.command_name == command.name)
                        .cloned()
                        .collect();
                    let plugin_info = PluginInfo {
                        title: command
                            .title
                            .clone()
                            .unwrap_or_else(|| command.name.clone()),
                        description: command
                            .description
                            .or_else(|| package_json.description.clone()),
                        plugin_title: package_json
                            .title
                            .clone()
                            .unwrap_or_else(|| plugin_dir_name.clone()),
                        plugin_name: package_json
                            .name
                            .clone()
                            .unwrap_or_else(|| plugin_dir_name.clone()),
                        command_name: command.name.clone(),
                        plugin_path: command_file_path.to_string_lossy().to_string(),
                        icon: command.icon.or_else(|| package_json.icon.clone()),
                        preferences: package_json.preferences.clone(),
                        command_preferences: command.preferences,
                        mode: command.mode,
                        author: package_json.author.clone(),
                        owner: package_json.owner.clone(),
                        compatibility_warnings: if warnings.is_empty() {
                            None
                        } else {
                            Some(warnings)
                        },
                        compatibility_score: Some(compatibility_metadata.compatibility_score),
                    };
                    plugins.push(plugin_info);
                } else {
                    tracing::warn!(
                        command = %command.name,
                        path = %command_file_path.display(),
                        "Command file not found"
                    );
                }
            }
        }
    }

    Ok(plugins)
}

#[tauri::command]
pub async fn install_extension(
    app: tauri::AppHandle,
    download_url: String,
    slug: String,
    force: bool,
) -> Result<InstallResult, String> {
    let extension_dir = get_extension_dir(&app, &slug)?;
    let content = download_archive(&download_url).await?;

    let heuristic_result = run_heuristic_checks(&content)?;
    if !heuristic_result.violations.is_empty() && !force {
        return Ok(InstallResult::RequiresConfirmation {
            violations: heuristic_result.violations.clone(),
        });
    }

    extract_archive(&content, &extension_dir)?;

    // Attempt to substitute macOS binaries with Linux equivalents
    if !heuristic_result.macho_binaries.is_empty() {
        match cli_substitutes::substitute_macos_binaries(
            &extension_dir,
            &heuristic_result.macho_binaries,
        )
        .await
        {
            Ok(substituted) => {
                if !substituted.is_empty() {
                    tracing::info!(
                        count = substituted.len(),
                        "Successfully substituted macOS binaries with Linux versions"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to substitute some binaries");
            }
        }
    }

    save_compatibility_metadata(&extension_dir, &heuristic_result.violations)?;

    Ok(InstallResult::Success)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityInfo {
    pub slug: String,
    pub compatibility_score: u8,
    pub warnings: Vec<HeuristicViolation>,
}

#[tauri::command]
pub fn get_extension_compatibility(
    app: tauri::AppHandle,
    slug: String,
) -> Result<CompatibilityInfo, String> {
    let extension_dir = get_extension_dir(&app, &slug)?;
    let metadata = load_compatibility_metadata(&extension_dir)?;

    Ok(CompatibilityInfo {
        slug,
        compatibility_score: metadata.compatibility_score,
        warnings: metadata.warnings,
    })
}

#[tauri::command]
pub fn get_all_extensions_compatibility(
    app: tauri::AppHandle,
) -> Result<Vec<CompatibilityInfo>, String> {
    let plugins_base_dir = get_extension_dir(&app, "")?;
    let mut results = Vec::new();

    if !plugins_base_dir.exists() {
        return Ok(results);
    }

    let plugin_dirs = fs::read_dir(plugins_base_dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir());

    for plugin_dir_entry in plugin_dirs {
        let plugin_dir = plugin_dir_entry.path();
        let slug = plugin_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        if let Ok(metadata) = load_compatibility_metadata(&plugin_dir) {
            results.push(CompatibilityInfo {
                slug,
                compatibility_score: metadata.compatibility_score,
                warnings: metadata.warnings,
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn uninstall_extension(app: tauri::AppHandle, slug: String) -> Result<(), String> {
    let extension_dir = get_extension_dir(&app, &slug)?;

    if !extension_dir.exists() {
        return Err(format!("Extension '{}' is not installed", slug));
    }

    fs::remove_dir_all(&extension_dir)
        .map_err(|e| format!("Failed to uninstall extension: {}", e))?;

    tracing::info!(slug = %slug, "Extension uninstalled successfully");
    Ok(())
}
