use serde::{Deserialize, Serialize};
use std::process::Command;

/// Power management commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerCommand {
    Shutdown,
    Restart,
    Sleep,
    Lock,
}

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeLevel {
    pub percentage: u8,
    pub is_muted: bool,
}

/// Execute a systemctl command
async fn execute_systemctl_command(action: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .arg(action)
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("systemctl {} failed: {}", action, stderr));
    }

    Ok(())
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Execute a power management command
#[tauri::command]
pub async fn execute_power_command(command: PowerCommand) -> Result<(), String> {
    tracing::info!("Executing power command: {:?}", command);

    match command {
        PowerCommand::Shutdown => execute_systemctl_command("poweroff").await,
        PowerCommand::Restart => execute_systemctl_command("reboot").await,
        PowerCommand::Sleep => execute_systemctl_command("suspend").await,
        PowerCommand::Lock => {
            // Detect desktop environment first and use DE-specific commands
            let de = std::env::var("XDG_CURRENT_DESKTOP")
                .unwrap_or_default()
                .to_lowercase();

            tracing::info!("Detected desktop environment: {}", de);

            // Try DE-specific lock commands first
            let lock_result = if de.contains("cinnamon") {
                tracing::info!("Using cinnamon-screensaver-command for lock");
                Command::new("cinnamon-screensaver-command")
                    .arg("-l")
                    .output()
            } else if de.contains("kde") || de.contains("plasma") {
                tracing::info!("Using qdbus for KDE lock");
                Command::new("qdbus")
                    .args(["org.kde.screensaver", "/ScreenSaver", "Lock"])
                    .output()
            } else if de.contains("xfce") {
                tracing::info!("Using xflock4 for XFCE lock");
                Command::new("xflock4").output()
            } else if de.contains("mate") {
                tracing::info!("Using mate-screensaver-command for MATE lock");
                Command::new("mate-screensaver-command").arg("-l").output()
            } else {
                // Fall back to loginctl for other DEs (GNOME, etc.)
                tracing::info!("Using loginctl for lock (generic)");
                Command::new("loginctl").arg("lock-session").output()
            };

            match lock_result {
                Ok(output) if output.status.success() => {
                    tracing::info!("Lock screen command succeeded");
                    Ok(())
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("Lock screen command failed: {}", stderr);
                    Err(format!("Failed to lock screen: {}", stderr))
                }
                Err(e) => {
                    tracing::error!("Failed to execute lock command: {}", e);
                    Err(format!("Failed to execute lock command: {}", e))
                }
            }
        }
    }
}

/// Set system volume (0-100%)
#[tauri::command]
pub async fn set_volume(level: u8) -> Result<(), String> {
    let level = level.min(100); // Clamp to 0-100
    tracing::info!("Setting volume to {}%", level);

    // Try pactl first (PulseAudio/PipeWire)
    if command_exists("pactl") {
        let output = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", level)])
            .output()
            .map_err(|e| format!("Failed to execute pactl: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    // Fallback to amixer (ALSA)
    if command_exists("amixer") {
        let output = Command::new("amixer")
            .args(["set", "Master", &format!("{}%", level)])
            .output()
            .map_err(|e| format!("Failed to execute amixer: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    Err("No audio system found. Install PulseAudio/PipeWire (pactl) or ALSA (amixer).".to_string())
}

/// Increase volume by 5%
#[tauri::command]
pub async fn volume_up() -> Result<(), String> {
    tracing::info!("Increasing volume");

    if command_exists("pactl") {
        let output = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", "+5%"])
            .output()
            .map_err(|e| format!("Failed to execute pactl: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    if command_exists("amixer") {
        let output = Command::new("amixer")
            .args(["set", "Master", "5%+"])
            .output()
            .map_err(|e| format!("Failed to execute amixer: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    Err("No audio system found.".to_string())
}

/// Decrease volume by 5%
#[tauri::command]
pub async fn volume_down() -> Result<(), String> {
    tracing::info!("Decreasing volume");

    if command_exists("pactl") {
        let output = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", "-5%"])
            .output()
            .map_err(|e| format!("Failed to execute pactl: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    if command_exists("amixer") {
        let output = Command::new("amixer")
            .args(["set", "Master", "5%-"])
            .output()
            .map_err(|e| format!("Failed to execute amixer: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    Err("No audio system found.".to_string())
}

/// Toggle mute
#[tauri::command]
pub async fn toggle_mute() -> Result<(), String> {
    tracing::info!("Toggling mute");

    if command_exists("pactl") {
        let output = Command::new("pactl")
            .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
            .output()
            .map_err(|e| format!("Failed to execute pactl: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    if command_exists("amixer") {
        let output = Command::new("amixer")
            .args(["set", "Master", "toggle"])
            .output()
            .map_err(|e| format!("Failed to execute amixer: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
    }

    Err("No audio system found.".to_string())
}

/// Get current volume level and mute status
#[tauri::command]
pub async fn get_volume() -> Result<VolumeLevel, String> {
    if command_exists("pactl") {
        // Get volume
        let volume_output = Command::new("pactl")
            .args(["get-sink-volume", "@DEFAULT_SINK@"])
            .output()
            .map_err(|e| format!("Failed to get volume: {}", e))?;

        let volume_str = String::from_utf8_lossy(&volume_output.stdout);

        // Parse percentage from output like "Volume: front-left: 65536 / 100% / 0.00 dB"
        let percentage = volume_str
            .split('/')
            .nth(1)
            .and_then(|s| s.trim().trim_end_matches('%').parse::<u8>().ok())
            .unwrap_or(50);

        // Get mute status
        let mute_output = Command::new("pactl")
            .args(["get-sink-mute", "@DEFAULT_SINK@"])
            .output()
            .map_err(|e| format!("Failed to get mute status: {}", e))?;

        let mute_str = String::from_utf8_lossy(&mute_output.stdout);
        let is_muted = mute_str.to_lowercase().contains("yes");

        return Ok(VolumeLevel {
            percentage,
            is_muted,
        });
    }

    if command_exists("amixer") {
        let output = Command::new("amixer")
            .args(["get", "Master"])
            .output()
            .map_err(|e| format!("Failed to get volume: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse output like "Front Left: Playback 65536 [100%] [on]"
        let percentage = output_str
            .split('[')
            .nth(1)
            .and_then(|s| s.split('%').next())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(50);

        let is_muted = output_str.to_lowercase().contains("[off]");

        return Ok(VolumeLevel {
            percentage,
            is_muted,
        });
    }

    Err("No audio system found.".to_string())
}

/// Empty the trash
#[tauri::command]
pub async fn empty_trash() -> Result<usize, String> {
    tracing::info!("Emptying trash");

    let trash_path = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".local/share/Trash/files");

    if !trash_path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in std::fs::read_dir(&trash_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        } else {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        count += 1;
    }

    tracing::info!("Emptied {} items from trash", count);
    Ok(count)
}

/// Eject a drive
#[tauri::command]
pub async fn eject_drive(device: String) -> Result<(), String> {
    tracing::info!("Ejecting drive: {}", device);

    // Unmount first
    let output = Command::new("udisksctl")
        .args(["unmount", "-b", &device])
        .output()
        .map_err(|e| format!("Failed to unmount: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to unmount device: {}", stderr));
    }

    // Try to power off (optional, may not be supported on all devices)
    let _ = Command::new("udisksctl")
        .args(["power-off", "-b", &device])
        .output();

    Ok(())
}
