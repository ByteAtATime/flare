use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use which::which;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub exec: String,
    pub keywords: Vec<String>,
    pub no_display: bool,
    pub path: PathBuf,
}

pub fn scan_applications() -> Vec<AppEntry> {
    let dirs = get_app_dirs();
    let current_desktop = env::var("XDG_CURRENT_DESKTOP").ok();

    let mut file_map: HashMap<String, PathBuf> = HashMap::new();

    for dir in dirs.iter().rev() {
        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "desktop")
            {
                let filename = entry.file_name().to_string_lossy().to_string();
                file_map.insert(filename, entry.path().to_path_buf());
            }
        }
    }

    let paths: Vec<PathBuf> = file_map.into_values().collect();

    let mut apps: Vec<AppEntry> = paths
        .par_iter()
        .filter_map(|path| parse_desktop_file(path, current_desktop.as_ref()))
        .collect();

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps
}

pub fn launch_application(app: &AppEntry) {
    let cmd_parts: Vec<&str> = app.exec.split_whitespace().collect();
    if let Some((cmd, args)) = cmd_parts.split_first() {
        let _ = Command::new(cmd).args(args).spawn();
    }
}

fn get_app_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(local_share) = dirs::data_local_dir() {
        dirs.push(local_share.join("applications"));
        dirs.push(local_share.join("flatpak/exports/share/applications"));
    }

    if let Ok(data_dirs) = env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            let path = PathBuf::from(dir).join("applications");
            dirs.push(path);
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
        dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));
    }

    dirs.into_iter().filter(|p| p.exists()).collect()
}

fn parse_desktop_file(path: &Path, current_desktop: Option<&String>) -> Option<AppEntry> {
    let conf = ini::Ini::load_from_file(path).ok()?;
    let section = conf.section(Some("Desktop Entry"))?;

    if section.get("Type")? != "Application" {
        return None;
    }

    let no_display = section
        .get("NoDisplay")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if no_display {
        return None;
    }

    if let Some(try_exec) = section.get("TryExec") {
        if which(try_exec).is_err() {
            return None;
        }
    }

    if let Some(desktop) = current_desktop {
        if let Some(only_show) = section.get("OnlyShowIn") {
            let allowed: Vec<&str> = only_show.split(';').filter(|s| !s.is_empty()).collect();
            if !allowed.contains(&desktop.as_str()) {
                return None;
            }
        }
        if let Some(not_show) = section.get("NotShowIn") {
            let forbidden: Vec<&str> = not_show.split(';').filter(|s| !s.is_empty()).collect();
            if forbidden.contains(&desktop.as_str()) {
                return None;
            }
        }
    }

    let name = section.get("Name")?.to_string();
    let icon = section.get("Icon").unwrap_or("").to_string();

    let raw_exec = section.get("Exec")?;
    let exec = clean_exec_code(raw_exec);

    let keywords = section
        .get("Keywords")
        .map(|k| k.split(';').map(|s| s.to_string()).collect())
        .unwrap_or_default();

    Some(AppEntry {
        id: path.file_name()?.to_string_lossy().to_string(),
        name,
        icon,
        exec,
        keywords,
        no_display,
        path: path.to_path_buf(),
    })
}

fn clean_exec_code(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<&str>>()
        .join(" ")
}
