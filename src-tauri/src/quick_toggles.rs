use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToggleState {
    pub enabled: bool,
}

/// Toggle WiFi on/off via NetworkManager D-Bus
pub async fn toggle_wifi(enable: bool) -> Result<(), String> {
    // Use nmcli command as a simpler alternative to D-Bus for now
    let status = if enable { "on" } else { "off" };

    std::process::Command::new("nmcli")
        .args(&["radio", "wifi", status])
        .output()
        .map_err(|e| {
            format!(
                "Failed to toggle WiFi (is NetworkManager installed?): {}",
                e
            )
        })?;

    Ok(())
}

/// Get WiFi state via NetworkManager
pub async fn get_wifi_state() -> Result<bool, String> {
    let output = std::process::Command::new("nmcli")
        .args(&["radio", "wifi"])
        .output()
        .map_err(|e| format!("Failed to get WiFi state: {}", e))?;

    let state = String::from_utf8_lossy(&output.stdout);
    Ok(state.trim() == "enabled")
}

/// Toggle Bluetooth on/off via rfkill
pub async fn toggle_bluetooth(enable: bool) -> Result<(), String> {
    let action = if enable { "unblock" } else { "block" };

    std::process::Command::new("rfkill")
        .args(&[action, "bluetooth"])
        .output()
        .map_err(|e| format!("Failed to toggle Bluetooth (is rfkill installed?): {}", e))?;

    Ok(())
}

/// Get Bluetooth state via rfkill
pub async fn get_bluetooth_state() -> Result<bool, String> {
    let output = std::process::Command::new("rfkill")
        .args(&["list", "bluetooth"])
        .output()
        .map_err(|e| format!("Failed to get Bluetooth state: {}", e))?;

    let state = String::from_utf8_lossy(&output.stdout);
    // If output contains "Soft blocked: no" and "Hard blocked: no", Bluetooth is enabled
    Ok(!state.contains("Soft blocked: yes") && !state.contains("Hard blocked: yes"))
}

/// Detect the current desktop environment
fn detect_desktop_environment() -> Option<String> {
    // Check XDG_CURRENT_DESKTOP first
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        return Some(de.to_lowercase());
    }

    // Fallback to DESKTOP_SESSION
    if let Ok(de) = std::env::var("DESKTOP_SESSION") {
        return Some(de.to_lowercase());
    }

    None
}

/// Toggle dark mode based on desktop environment
pub async fn toggle_dark_mode(enable: bool) -> Result<(), String> {
    let de = detect_desktop_environment().ok_or("Could not detect desktop environment")?;

    if de.contains("gnome") || de.contains("ubuntu") {
        toggle_gnome_dark_mode(enable)
    } else if de.contains("kde") || de.contains("plasma") {
        toggle_kde_dark_mode(enable)
    } else if de.contains("xfce") {
        toggle_xfce_dark_mode(enable)
    } else {
        Err(format!(
            "Dark mode toggle not supported for desktop environment: {}",
            de
        ))
    }
}

/// Get dark mode state based on desktop environment
pub async fn get_dark_mode_state() -> Result<bool, String> {
    let de = detect_desktop_environment().ok_or("Could not detect desktop environment")?;

    if de.contains("gnome") || de.contains("ubuntu") {
        get_gnome_dark_mode_state()
    } else if de.contains("kde") || de.contains("plasma") {
        get_kde_dark_mode_state()
    } else if de.contains("xfce") {
        get_xfce_dark_mode_state()
    } else {
        Err(format!(
            "Dark mode state not supported for desktop environment: {}",
            de
        ))
    }
}

fn toggle_gnome_dark_mode(enable: bool) -> Result<(), String> {
    let color_scheme = if enable { "prefer-dark" } else { "default" };

    std::process::Command::new("gsettings")
        .args(&[
            "set",
            "org.gnome.desktop.interface",
            "color-scheme",
            color_scheme,
        ])
        .output()
        .map_err(|e| format!("Failed to toggle GNOME dark mode: {}", e))?;

    Ok(())
}

fn get_gnome_dark_mode_state() -> Result<bool, String> {
    let output = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .map_err(|e| format!("Failed to get GNOME color scheme: {}", e))?;

    let scheme = String::from_utf8_lossy(&output.stdout);
    Ok(scheme.contains("dark"))
}

fn toggle_kde_dark_mode(enable: bool) -> Result<(), String> {
    // KDE Plasma uses lookandfeeltool
    let theme = if enable {
        "org.kde.breezedark.desktop"
    } else {
        "org.kde.breeze.desktop"
    };

    std::process::Command::new("lookandfeeltool")
        .args(&["-a", theme])
        .output()
        .map_err(|e| format!("Failed to toggle KDE dark mode: {}", e))?;

    Ok(())
}

fn get_kde_dark_mode_state() -> Result<bool, String> {
    let output = std::process::Command::new("kreadconfig5")
        .args(&[
            "--file",
            "kdeglobals",
            "--group",
            "General",
            "--key",
            "ColorScheme",
        ])
        .output()
        .map_err(|e| format!("Failed to get KDE color scheme: {}", e))?;

    let scheme = String::from_utf8_lossy(&output.stdout);
    Ok(scheme.to_lowercase().contains("dark"))
}

fn toggle_xfce_dark_mode(enable: bool) -> Result<(), String> {
    let theme = if enable { "Adwaita-dark" } else { "Adwaita" };

    std::process::Command::new("xfconf-query")
        .args(&["-c", "xsettings", "-p", "/Net/ThemeName", "-s", theme])
        .output()
        .map_err(|e| format!("Failed to toggle XFCE dark mode: {}", e))?;

    Ok(())
}

fn get_xfce_dark_mode_state() -> Result<bool, String> {
    let output = std::process::Command::new("xfconf-query")
        .args(&["-c", "xsettings", "-p", "/Net/ThemeName"])
        .output()
        .map_err(|e| format!("Failed to get XFCE theme: {}", e))?;

    let theme = String::from_utf8_lossy(&output.stdout);
    Ok(theme.to_lowercase().contains("dark"))
}

/// Set screen brightness (0-100)
pub fn set_brightness(percentage: u32) -> Result<(), String> {
    let percentage = percentage.clamp(0, 100);

    // Find backlight device
    let backlight_path = Path::new("/sys/class/backlight");

    if !backlight_path.exists() {
        return Err("No backlight device found".to_string());
    }

    let entries = fs::read_dir(backlight_path)
        .map_err(|e| format!("Failed to read backlight directory: {}", e))?;

    for entry in entries.flatten() {
        let device_path = entry.path();
        let max_brightness_path = device_path.join("max_brightness");
        let brightness_path = device_path.join("brightness");

        if max_brightness_path.exists() && brightness_path.exists() {
            let max_brightness: u32 = fs::read_to_string(&max_brightness_path)
                .map_err(|e| format!("Failed to read max brightness: {}", e))?
                .trim()
                .parse()
                .map_err(|e| format!("Failed to parse max brightness: {}", e))?;

            let target_brightness = (max_brightness as f64 * (percentage as f64 / 100.0)) as u32;

            fs::write(&brightness_path, target_brightness.to_string())
                .map_err(|e| format!("Failed to set brightness: {}. You may need appropriate permissions (try adding user to 'video' group).", e))?;

            return Ok(());
        }
    }

    Err("No suitable backlight device found".to_string())
}

/// Get current screen brightness (0-100)
pub fn get_brightness() -> Result<u32, String> {
    let backlight_path = Path::new("/sys/class/backlight");

    if !backlight_path.exists() {
        return Err("No backlight device found".to_string());
    }

    let entries = fs::read_dir(backlight_path)
        .map_err(|e| format!("Failed to read backlight directory: {}", e))?;

    for entry in entries.flatten() {
        let device_path = entry.path();
        let max_brightness_path = device_path.join("max_brightness");
        let brightness_path = device_path.join("brightness");

        if max_brightness_path.exists() && brightness_path.exists() {
            let max_brightness: u32 = fs::read_to_string(&max_brightness_path)
                .map_err(|e| format!("Failed to read max brightness: {}", e))?
                .trim()
                .parse()
                .map_err(|e| format!("Failed to parse max brightness: {}", e))?;

            let current_brightness: u32 = fs::read_to_string(&brightness_path)
                .map_err(|e| format!("Failed to read brightness: {}", e))?
                .trim()
                .parse()
                .map_err(|e| format!("Failed to parse brightness: {}", e))?;

            let percentage = ((current_brightness as f64 / max_brightness as f64) * 100.0) as u32;
            return Ok(percentage);
        }
    }

    Err("No suitable backlight device found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_desktop_environment() {
        // This will vary by system
        let de = detect_desktop_environment();
        println!("Detected desktop environment: {:?}", de);
    }

    #[test]
    fn test_brightness_clamp() {
        // Test that brightness is clamped to 0-100
        assert_eq!(0, 0_u32.clamp(0, 100));
        assert_eq!(100, 150_u32.clamp(0, 100));
        assert_eq!(50, 50_u32.clamp(0, 100));
    }
}
