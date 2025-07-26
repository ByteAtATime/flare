use std::process::Command;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    name: String,
    path: String,
    bundle_id: Option<String>,
}

#[tauri::command]
pub fn trash(paths: Vec<String>) -> Result<(), String> {
    trash::delete_all(paths).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let path = std::path::Path::new(&path);
        let parent = path.parent().unwrap_or(path).as_os_str();
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_applications(
    app: tauri::AppHandle,
    _path: Option<String>,
) -> Result<Vec<Application>, String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
            set output to ""
            set app_paths to paragraphs of (do shell script "mdfind 'kMDItemContentType == \"com.apple.application-bundle\"' -onlyin /Applications -onlyin /System/Applications -onlyin ~/Applications")
            repeat with app_path in app_paths
                if app_path is not "" then
                    try
                        set app_info to info for (app_path as POSIX file)
                        set app_name to name of app_info
                        set bundle_id to bundle identifier of app_info
                        set output to output & app_name & "%%" & app_path & "%%" & bundle_id & "\n"
                    on error
                        -- ignore apps we can't get info for
                    end try
                end if
            end repeat
            return output
        "#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result_str = String::from_utf8_lossy(&output.stdout);
        let apps = result_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split("%%").collect();
                if parts.len() == 3 {
                    Some(Application {
                        name: parts[0].to_string(),
                        path: parts[1].to_string(),
                        bundle_id: Some(parts[2].to_string()),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(apps)
    }

    #[cfg(target_os = "linux")]
    {
        Ok(crate::get_installed_apps(app)
            .into_iter()
            .map(|app| Application {
                name: app.name,
                path: app.exec.unwrap_or_default(),
                bundle_id: None,
            })
            .collect())
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let uninstall = hklm
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
            .map_err(|e| e.to_string())?;
        let mut apps = Vec::new();

        for key in uninstall.enum_keys().filter_map(Result::ok) {
            if let Ok(subkey) = uninstall.open_subkey(key) {
                if let (Ok(name), Ok(path)) = (
                    subkey.get_value("DisplayName"),
                    subkey.get_value("InstallLocation"),
                ) {
                    let name_str: String = name;
                    let path_str: String = path;
                    if !name_str.is_empty() && !path_str.is_empty() {
                        apps.push(Application {
                            name: name_str,
                            path: path_str,
                            bundle_id: None,
                        });
                    }
                }
            }
        }
        Ok(apps)
    }
}

#[tauri::command]
pub fn get_default_application(path: String) -> Result<Application, String> {
    Err(format!(
        "get_default_application for '{}' is not yet implemented for this platform.",
        path
    ))
}

#[tauri::command]
pub fn get_frontmost_application() -> Result<Application, String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
            tell application "System Events"
                set front_app to first application process whose frontmost is true
                set app_path to (path of application file of front_app)
                set app_name to (name of front_app)
                set bundle_id to (bundle identifier of front_app)
                return app_name & "%%" & app_path & "%%" & bundle_id
            end tell
        "#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result_str.trim().split("%%").collect();
        if parts.len() == 3 {
            Ok(Application {
                name: parts[0].to_string(),
                path: parts[1].to_string(),
                bundle_id: Some(parts[2].to_string()),
            })
        } else {
            Err("Could not determine frontmost application".to_string())
        }
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        Err("get_frontmost_application is not yet implemented for this platform.".to_string())
    }
}
