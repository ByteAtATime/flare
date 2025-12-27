use std::fs;
use std::path::PathBuf;

/// Enable or disable auto-start on login (Linux XDG standard)
pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let autostart_dir = get_autostart_dir()?;
        let desktop_file_path = autostart_dir.join("flareup.desktop");

        if enabled {
            // Create autostart directory if it doesn't exist
            fs::create_dir_all(&autostart_dir)
                .map_err(|e| format!("Failed to create autostart directory: {}", e))?;

            // Get the executable path
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;

            // Create desktop entry content
            let desktop_entry = format!(
                "[Desktop Entry]\n\
                Type=Application\n\
                Name=Flareup\n\
                Comment=Raycast for Linux\n\
                Exec={}\n\
                Terminal=false\n\
                Categories=Utility;\n\
                X-GNOME-Autostart-enabled=true\n",
                exe_path.display()
            );

            // Write the desktop file
            fs::write(&desktop_file_path, desktop_entry)
                .map_err(|e| format!("Failed to write autostart file: {}", e))?;

            tracing::info!("Auto-start enabled");
            Ok(())
        } else {
            // Remove the desktop file if it exists
            if desktop_file_path.exists() {
                fs::remove_file(&desktop_file_path)
                    .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
                tracing::info!("Auto-start disabled");
            }
            Ok(())
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Auto-start is currently only supported on Linux".to_string())
    }
}

/// Check if auto-start is currently enabled
pub fn is_auto_start_enabled() -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        let autostart_dir = get_autostart_dir()?;
        let desktop_file_path = autostart_dir.join("flareup.desktop");
        Ok(desktop_file_path.exists())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
fn get_autostart_dir() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;

    // Check XDG_CONFIG_HOME first, fallback to ~/.config
    let config_home =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    Ok(PathBuf::from(config_home).join("autostart"))
}

// Tauri command
#[tauri::command]
pub fn set_auto_start_enabled(enabled: bool) -> Result<(), String> {
    set_auto_start(enabled)
}

#[tauri::command]
pub fn get_auto_start_enabled() -> Result<bool, String> {
    is_auto_start_enabled()
}
