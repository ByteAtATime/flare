use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Provides Linux equivalents for macOS-specific APIs used in Raycast extensions
/// This module helps bridge the gap between macOS and Linux for extension compatibility

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Enhanced AppleScript command types for better parsing and execution
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppleScriptCommand {
    DoShellScript {
        command: String,
        needs_sudo: bool,
    },
    TellApplication {
        app: String,
        command: AppCommand,
    },
    DisplayNotification {
        message: String,
        title: String,
    },
    SetVolume {
        volume: i32,
    },
    OpenLocation {
        location: String,
    },
    Keystroke {
        text: String,
        modifiers: Vec<Modifier>,
    },
    KeyCode {
        code: i32,
        modifiers: Vec<Modifier>,
    },
    Click {
        x: Option<i32>,
        y: Option<i32>,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    SetClipboard {
        text: String,
    },
    GetClipboard,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppCommand {
    Activate,
    Quit,
    Open { path: String },
    GetURL,                             // Browser: get current tab URL
    ExecuteJavaScript { code: String }, // Browser: execute JS
    Reveal { path: String },            // Finder: reveal file
    GetSelection,                       // Finder: get selected files
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Command, // Super/Meta key on Linux
    Control,
    Option, // Alt key on Linux
    Shift,
}

#[derive(Debug, Clone)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

/// Translates macOS paths to Linux equivalents
pub struct PathShim;

impl PathShim {
    /// Maps common macOS paths to their Linux equivalents
    pub fn translate_path(macos_path: &str) -> String {
        // Handle /Applications/ paths
        if macos_path.starts_with("/Applications/") {
            // Try to find the application in common Linux locations
            let app_name = macos_path
                .strip_prefix("/Applications/")
                .unwrap_or(macos_path)
                .trim_end_matches(".app")
                .to_lowercase();

            // Return the most likely Linux equivalent
            // Extensions will need to use the desktop file system instead
            return format!("/usr/share/applications/{}.desktop", app_name);
        }

        // Handle /Library/ paths
        if macos_path.starts_with("/Library/") {
            let rest = macos_path.strip_prefix("/Library/").unwrap_or("");
            return format!("/usr/lib/{}", rest);
        }

        // Handle ~/Library/ paths
        if macos_path.starts_with("~/Library/") {
            let rest = macos_path.strip_prefix("~/Library/").unwrap_or("");
            // Map to XDG directories where appropriate
            if rest.starts_with("Application Support/") {
                let app_rest = rest.strip_prefix("Application Support/").unwrap_or("");
                return format!("~/.local/share/{}", app_rest);
            }
            if rest.starts_with("Preferences/") {
                let pref_rest = rest.strip_prefix("Preferences/").unwrap_or("");
                return format!("~/.config/{}", pref_rest);
            }
            return format!("~/.local/lib/{}", rest);
        }

        // Handle /Users/ paths
        if macos_path.starts_with("/Users/") {
            return macos_path.replace("/Users/", "/home/");
        }

        // Return unchanged if no translation needed
        macos_path.to_string()
    }

    /// Expands ~ in paths to the actual home directory
    pub fn expand_home(path: &str) -> PathBuf {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(path.strip_prefix("~/").unwrap_or(path));
            }
        }
        PathBuf::from(path)
    }
}

/// Provides shims for AppleScript functionality
pub struct AppleScriptShim;

impl AppleScriptShim {
    /// Attempts to translate and execute common AppleScript commands
    pub fn run_apple_script(script: &str) -> ShimResult {
        // Parse common AppleScript patterns and translate to Linux equivalents

        // Pattern: do shell script
        if let Some((command, needs_sudo)) = Self::extract_shell_script(script) {
            return Self::run_shell_script(&command, needs_sudo);
        }

        // Pattern: open location
        if let Some(location) = Self::extract_open_location(script) {
            return Self::open_location(&location);
        }

        // Pattern: tell application "AppName" to activate
        if let Some(app_name) = Self::extract_activate_app(script) {
            return Self::activate_application(&app_name);
        }

        // Pattern: tell application "AppName" to quit
        if let Some(app_name) = Self::extract_quit_app(script) {
            return Self::quit_application(&app_name);
        }

        // Pattern: display notification
        if let Some((title, message)) = Self::extract_notification(script) {
            return Self::show_notification(&title, &message);
        }

        // Pattern: set volume
        if let Some(volume) = Self::extract_set_volume(script) {
            return Self::set_system_volume(volume);
        }

        // Pattern: set clipboard
        if let Some(text) = Self::extract_set_clipboard(script) {
            return Self::set_clipboard(&text);
        }

        // Pattern: get clipboard
        if Self::is_get_clipboard(script) {
            return Self::get_clipboard();
        }

        // Pattern: keystroke
        if let Some((text, modifiers)) = Self::extract_keystroke(script) {
            return Self::simulate_keystroke(&text, &modifiers);
        }

        // Pattern: key code
        if let Some((code, modifiers)) = Self::extract_keycode(script) {
            return Self::simulate_keycode(code, &modifiers);
        }
        // If we can't translate, return an error
        ShimResult {
            success: false,
            output: None,
            error: Some(format!(
                "AppleScript pattern not supported on Linux. Script: {}",
                script
            )),
        }
    }

    // ========== NEW PRIORITY 1 PARSERS ==========

    fn extract_shell_script(script: &str) -> Option<(String, bool)> {
        // Match: do shell script "command"
        // Also match: do shell script "command" with administrator privileges
        let pattern = r#"do shell script "([^"]+)"(?:\s+with administrator privileges)?"#;
        if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
            let command = caps.get(1)?.as_str().to_string();
            let needs_sudo = script.contains("with administrator privileges");
            return Some((command, needs_sudo));
        }
        None
    }

    fn extract_open_location(script: &str) -> Option<String> {
        // Match various open patterns
        let patterns = [
            r#"open location "([^"]+)""#,
            r#"open "([^"]+)""#,
            r#"tell application "Finder" to open "([^"]+)""#,
        ];

        for pattern in &patterns {
            if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_set_clipboard(script: &str) -> Option<String> {
        // Match: set the clipboard to "text"
        let pattern = r#"set the clipboard to "([^"]+)""#;
        regex::Regex::new(pattern)
            .ok()?
            .captures(script)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }

    fn is_get_clipboard(script: &str) -> bool {
        // Match "get the clipboard" but not "set the clipboard"
        script.contains("get the clipboard")
            || (script.contains("the clipboard") && !script.contains("set the clipboard"))
    }

    fn extract_keystroke(script: &str) -> Option<(String, Vec<Modifier>)> {
        // Match: tell application "System Events" to keystroke "text"
        // Also match: tell application "System Events" to keystroke "text" using {command down, shift down}
        let pattern = r#"keystroke "([^"]+)"(?:\s+using\s+\{([^}]+)\})?"#;

        if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
            let text = caps.get(1)?.as_str().to_string();
            let modifiers = if let Some(mods_str) = caps.get(2) {
                Self::parse_modifiers(mods_str.as_str())
            } else {
                Vec::new()
            };
            return Some((text, modifiers));
        }
        None
    }

    fn extract_keycode(script: &str) -> Option<(i32, Vec<Modifier>)> {
        // Match: tell application "System Events" to key code 36
        // Also match with modifiers: key code 36 using {command down}
        let pattern = r"key code (\d+)(?:\s+using\s+\{([^}]+)\})?";

        if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
            let code = caps.get(1)?.as_str().parse().ok()?;
            let modifiers = if let Some(mods_str) = caps.get(2) {
                Self::parse_modifiers(mods_str.as_str())
            } else {
                Vec::new()
            };
            return Some((code, modifiers));
        }
        None
    }

    fn parse_modifiers(mods_str: &str) -> Vec<Modifier> {
        let mut modifiers = Vec::new();

        if mods_str.contains("command down") || mods_str.contains("command_down") {
            modifiers.push(Modifier::Command);
        }
        if mods_str.contains("control down") || mods_str.contains("control_down") {
            modifiers.push(Modifier::Control);
        }
        if mods_str.contains("option down")
            || mods_str.contains("option_down")
            || mods_str.contains("alt down")
        {
            modifiers.push(Modifier::Option);
        }
        if mods_str.contains("shift down") || mods_str.contains("shift_down") {
            modifiers.push(Modifier::Shift);
        }

        modifiers
    }

    // ========== NEW PRIORITY 1 EXECUTORS ==========

    fn run_shell_script(command: &str, needs_sudo: bool) -> ShimResult {
        let mut cmd = if needs_sudo {
            let mut c = Command::new("pkexec");
            c.arg("sh").arg("-c").arg(command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                ShimResult {
                    success: output.status.success(),
                    output: if !stdout.is_empty() {
                        Some(stdout)
                    } else {
                        None
                    },
                    error: if !stderr.is_empty() {
                        Some(stderr)
                    } else {
                        None
                    },
                }
            }
            Err(e) => ShimResult {
                success: false,
                output: None,
                error: Some(format!("Failed to execute shell script: {}", e)),
            },
        }
    }

    fn open_location(location: &str) -> ShimResult {
        // Handle both URLs and file paths
        let location_expanded = if location.starts_with("file://") {
            PathShim::expand_home(&location[7..])
                .to_string_lossy()
                .to_string()
        } else if !location.starts_with("http://") && !location.starts_with("https://") {
            PathShim::expand_home(location)
                .to_string_lossy()
                .to_string()
        } else {
            location.to_string()
        };

        match Command::new("xdg-open").arg(&location_expanded).output() {
            Ok(output) if output.status.success() => ShimResult {
                success: true,
                output: Some(format!("Opened: {}", location)),
                error: None,
            },
            Ok(output) => ShimResult {
                success: false,
                output: None,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            },
            Err(e) => ShimResult {
                success: false,
                output: None,
                error: Some(format!("Failed to open location: {}", e)),
            },
        }
    }

    fn set_clipboard(text: &str) -> ShimResult {
        // Use wl-copy for Wayland or xclip for X11
        let display_server = Self::detect_display_server();

        let result = match display_server {
            DisplayServer::Wayland => Command::new("wl-copy").arg(text).output(),
            DisplayServer::X11 | DisplayServer::Unknown => {
                // Try xclip first
                let xclip_result = Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .arg("-i")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(mut stdin) = child.stdin.take() {
                            stdin.write_all(text.as_bytes())?;
                        }
                        child.wait_with_output()
                    });

                if xclip_result.is_ok() {
                    xclip_result
                } else {
                    // Fallback to xsel
                    Command::new("xsel")
                        .arg("--clipboard")
                        .arg("--input")
                        .arg(text)
                        .output()
                }
            }
        };

        match result {
            Ok(output) if output.status.success() => ShimResult {
                success: true,
                output: Some("Clipboard updated".to_string()),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some(
                    "Failed to set clipboard. Install wl-copy (Wayland) or xclip/xsel (X11)"
                        .to_string(),
                ),
            },
        }
    }

    fn get_clipboard() -> ShimResult {
        let display_server = Self::detect_display_server();

        let result = match display_server {
            DisplayServer::Wayland => Command::new("wl-paste").output(),
            DisplayServer::X11 | DisplayServer::Unknown => {
                // Try xclip first
                let xclip_result = Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .arg("-o")
                    .output();

                if xclip_result.is_ok() {
                    xclip_result
                } else {
                    // Fallback to xsel
                    Command::new("xsel")
                        .arg("--clipboard")
                        .arg("--output")
                        .output()
                }
            }
        };

        match result {
            Ok(output) if output.status.success() => ShimResult {
                success: true,
                output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some(
                    "Failed to get clipboard. Install wl-paste (Wayland) or xclip/xsel (X11)"
                        .to_string(),
                ),
            },
        }
    }

    fn detect_display_server() -> DisplayServer {
        // Check if we're running on Wayland or X11
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            DisplayServer::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            DisplayServer::X11
        } else {
            DisplayServer::Unknown
        }
    }

    // ========== NEW PRIORITY 2 EXECUTORS (GUI AUTOMATION) ==========

    fn simulate_keystroke(text: &str, modifiers: &[Modifier]) -> ShimResult {
        let display_server = Self::detect_display_server();

        match display_server {
            DisplayServer::Wayland => Self::simulate_keystroke_wayland(text, modifiers),
            DisplayServer::X11 => Self::simulate_keystroke_x11(text, modifiers),
            DisplayServer::Unknown => ShimResult {
                success: false,
                output: None,
                error: Some("Cannot detect display server (X11/Wayland)".to_string()),
            },
        }
    }

    fn simulate_keystroke_x11(text: &str, modifiers: &[Modifier]) -> ShimResult {
        // Build xdotool command
        let mut cmd = Command::new("xdotool");

        if modifiers.is_empty() {
            // Simple text typing
            cmd.arg("type").arg("--").arg(text);
        } else {
            // Key combination
            let modifier_keys = Self::modifiers_to_x11_keys(modifiers);
            let key_combo = if text.len() == 1 {
                format!("{}+{}", modifier_keys, text)
            } else {
                // If text is multi-char, treat as key name
                format!("{}+{}", modifier_keys, text)
            };
            cmd.arg("key").arg("--").arg(key_combo);
        }

        match cmd.output() {
            Ok(output) if output.status.success() => ShimResult {
                success: true,
                output: Some("Keystroke simulated".to_string()),
                error: None,
            },
            Ok(output) => ShimResult {
                success: false,
                output: None,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            },
            Err(_) => ShimResult {
                success: false,
                output: None,
                error: Some(
                    "Failed to execute xdotool. Install with: sudo apt install xdotool".to_string(),
                ),
            },
        }
    }

    fn simulate_keystroke_wayland(text: &str, modifiers: &[Modifier]) -> ShimResult {
        // Build ydotool command
        let mut cmd = Command::new("ydotool");

        if modifiers.is_empty() {
            // Simple text typing
            cmd.arg("type").arg(text);
        } else {
            // Key combination - ydotool uses different approach
            let modifier_keys = Self::modifiers_to_wayland_keys(modifiers);
            cmd.arg("key").arg(format!("{}:{}", modifier_keys, text));
        }

        match cmd.output() {
            Ok(output) if output.status.success() => ShimResult {
                success: true,
                output: Some("Keystroke simulated".to_string()),
                error: None,
            },
            Ok(output) => ShimResult {
                success: false,
                output: None,
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            },
            Err(_) => ShimResult {
                success: false,
                output: None,
                error: Some(
                    "Failed to execute ydotool. Install with: sudo apt install ydotool".to_string(),
                ),
            },
        }
    }

    fn simulate_keycode(code: i32, modifiers: &[Modifier]) -> ShimResult {
        let display_server = Self::detect_display_server();

        // Map macOS key codes to Linux equivalents
        let linux_key = Self::macos_keycode_to_linux(code);

        match display_server {
            DisplayServer::X11 => {
                let modifier_keys = Self::modifiers_to_x11_keys(modifiers);
                let key_combo = if modifier_keys.is_empty() {
                    linux_key.to_string()
                } else {
                    format!("{}+{}", modifier_keys, linux_key)
                };

                match Command::new("xdotool")
                    .arg("key")
                    .arg("--")
                    .arg(key_combo)
                    .output()
                {
                    Ok(output) if output.status.success() => ShimResult {
                        success: true,
                        output: Some("Key code simulated".to_string()),
                        error: None,
                    },
                    _ => ShimResult {
                        success: false,
                        output: None,
                        error: Some("Failed to simulate key code".to_string()),
                    },
                }
            }
            DisplayServer::Wayland => ShimResult {
                success: false,
                output: None,
                error: Some("Key code simulation not yet supported on Wayland".to_string()),
            },
            DisplayServer::Unknown => ShimResult {
                success: false,
                output: None,
                error: Some("Cannot detect display server".to_string()),
            },
        }
    }

    fn modifiers_to_x11_keys(modifiers: &[Modifier]) -> String {
        let mut keys = Vec::new();
        for modifier in modifiers {
            match modifier {
                Modifier::Command => keys.push("super"),
                Modifier::Control => keys.push("ctrl"),
                Modifier::Option => keys.push("alt"),
                Modifier::Shift => keys.push("shift"),
            }
        }
        keys.join("+")
    }

    fn modifiers_to_wayland_keys(modifiers: &[Modifier]) -> String {
        let mut keys = Vec::new();
        for modifier in modifiers {
            match modifier {
                Modifier::Command => keys.push("125"), // Left Super key code
                Modifier::Control => keys.push("29"),  // Left Ctrl key code
                Modifier::Option => keys.push("56"),   // Left Alt key code
                Modifier::Shift => keys.push("42"),    // Left Shift key code
            }
        }
        keys.join(":")
    }

    fn macos_keycode_to_linux(code: i32) -> String {
        // Map common macOS key codes to Linux key names
        match code {
            36 => "Return".to_string(),
            51 => "BackSpace".to_string(),
            53 => "Escape".to_string(),
            48 => "Tab".to_string(),
            49 => "space".to_string(),
            123 => "Left".to_string(),
            124 => "Right".to_string(),
            125 => "Down".to_string(),
            126 => "Up".to_string(),
            116 => "Page_Up".to_string(),
            121 => "Page_Down".to_string(),
            115 => "Home".to_string(),
            119 => "End".to_string(),
            117 => "Delete".to_string(),
            _ => format!("KEY_{}", code), // Fallback for unknown codes
        }
    }
    // ========== EXISTING PARSERS (keeping for backwards compatibility) ==========
    fn extract_activate_app(script: &str) -> Option<String> {
        // Match: tell application "AppName" to activate
        let patterns = [
            r#"tell application "([^"]+)" to activate"#,
            r#"activate application "([^"]+)""#,
        ];

        for pattern in &patterns {
            if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_quit_app(script: &str) -> Option<String> {
        // Match: tell application "AppName" to quit
        let pattern = r#"tell application "([^"]+)" to quit"#;
        regex::Regex::new(pattern)
            .ok()?
            .captures(script)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }

    fn extract_notification(script: &str) -> Option<(String, String)> {
        // Match: display notification "message" with title "title"
        let pattern = r#"display notification "([^"]+)"(?:\s+with title "([^"]+)")?"#;
        let caps = regex::Regex::new(pattern).ok()?.captures(script)?;

        let message = caps.get(1)?.as_str().to_string();
        let title = caps
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Notification".to_string());

        Some((title, message))
    }

    fn extract_set_volume(script: &str) -> Option<i32> {
        // Match: set volume N or set volume output volume N
        let patterns = [r"set volume (\d+)", r"set volume output volume (\d+)"];

        for pattern in &patterns {
            if let Some(caps) = regex::Regex::new(pattern).ok()?.captures(script) {
                return caps.get(1)?.as_str().parse().ok();
            }
        }
        None
    }

    fn activate_application(app_name: &str) -> ShimResult {
        // Try to launch the application using the desktop file
        let desktop_name = app_name.to_lowercase();

        // Try using gtk-launch (works on most desktop environments)
        let output = Command::new("gtk-launch").arg(&desktop_name).output();

        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Activated application: {}", app_name)),
                error: None,
            },
            _ => {
                // Fallback: try xdg-open
                let fallback = Command::new("xdg-open").arg(&desktop_name).output();

                match fallback {
                    Ok(out) if out.status.success() => ShimResult {
                        success: true,
                        output: Some(format!("Activated application: {}", app_name)),
                        error: None,
                    },
                    _ => ShimResult {
                        success: false,
                        output: None,
                        error: Some(format!("Failed to activate application: {}", app_name)),
                    },
                }
            }
        }
    }

    fn quit_application(app_name: &str) -> ShimResult {
        // Try to quit the application using pkill
        let process_name = app_name.to_lowercase();

        let output = Command::new("pkill").arg("-f").arg(&process_name).output();

        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Quit application: {}", app_name)),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some(format!("Failed to quit application: {}", app_name)),
            },
        }
    }

    fn show_notification(title: &str, message: &str) -> ShimResult {
        // Use notify-send for freedesktop notifications
        let output = Command::new("notify-send").arg(title).arg(message).output();

        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some("Notification sent".to_string()),
                error: None,
            },
            _ => ShimResult {
                success: false,
                output: None,
                error: Some("Failed to send notification".to_string()),
            },
        }
    }

    fn set_system_volume(volume: i32) -> ShimResult {
        // Clamp volume to 0-100
        let vol = volume.clamp(0, 100);

        // Try using pactl (PulseAudio/PipeWire)
        let output = Command::new("pactl")
            .arg("set-sink-volume")
            .arg("@DEFAULT_SINK@")
            .arg(format!("{}%", vol))
            .output();

        match output {
            Ok(out) if out.status.success() => ShimResult {
                success: true,
                output: Some(format!("Set volume to {}%", vol)),
                error: None,
            },
            _ => {
                // Fallback: try amixer (ALSA)
                let fallback = Command::new("amixer")
                    .arg("set")
                    .arg("Master")
                    .arg(format!("{}%", vol))
                    .output();

                match fallback {
                    Ok(out) if out.status.success() => ShimResult {
                        success: true,
                        output: Some(format!("Set volume to {}%", vol)),
                        error: None,
                    },
                    _ => ShimResult {
                        success: false,
                        output: None,
                        error: Some("Failed to set volume".to_string()),
                    },
                }
            }
        }
    }
}

/// System API shims for common macOS system operations
pub struct SystemShim;

impl SystemShim {
    /// Get system information that might be requested by extensions
    pub fn get_system_info() -> HashMap<String, String> {
        let mut info = HashMap::new();

        // Platform
        info.insert("platform".to_string(), "linux".to_string());

        // Architecture
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if let Ok(arch) = String::from_utf8(output.stdout) {
                info.insert("arch".to_string(), arch.trim().to_string());
            }
        }

        // Hostname
        if let Ok(output) = Command::new("hostname").output() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                info.insert("hostname".to_string(), hostname.trim().to_string());
            }
        }

        // Desktop environment
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            info.insert("desktop_environment".to_string(), de);
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_translation_applications() {
        assert_eq!(
            PathShim::translate_path("/Applications/Safari.app"),
            "/usr/share/applications/safari.desktop"
        );
    }

    #[test]
    fn test_path_translation_library() {
        assert_eq!(
            PathShim::translate_path("/Library/Frameworks/Something"),
            "/usr/lib/Frameworks/Something"
        );
    }

    #[test]
    fn test_path_translation_users() {
        assert_eq!(
            PathShim::translate_path("/Users/john/Documents"),
            "/home/john/Documents"
        );
    }

    #[test]
    fn test_path_translation_user_library() {
        assert_eq!(
            PathShim::translate_path("~/Library/Application Support/MyApp"),
            "~/.local/share/MyApp"
        );
    }

    #[test]
    fn test_extract_activate_app() {
        let script = r#"tell application "Safari" to activate"#;
        assert_eq!(
            AppleScriptShim::extract_activate_app(script),
            Some("Safari".to_string())
        );
    }

    #[test]
    fn test_extract_notification() {
        let script = r#"display notification "Hello World" with title "Test""#;
        assert_eq!(
            AppleScriptShim::extract_notification(script),
            Some(("Test".to_string(), "Hello World".to_string()))
        );
    }

    // ========== NEW PRIORITY 1 TESTS ==========

    #[test]
    fn test_extract_shell_script() {
        let script = r#"do shell script "echo hello""#;
        assert_eq!(
            AppleScriptShim::extract_shell_script(script),
            Some(("echo hello".to_string(), false))
        );
    }

    #[test]
    fn test_extract_shell_script_with_sudo() {
        let script = r#"do shell script "whoami" with administrator privileges"#;
        assert_eq!(
            AppleScriptShim::extract_shell_script(script),
            Some(("whoami".to_string(), true))
        );
    }

    #[test]
    fn test_run_shell_script() {
        let result = AppleScriptShim::run_shell_script("echo hello", false);
        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.output.unwrap().contains("hello"));
    }

    #[test]
    fn test_extract_open_location_url() {
        let script = r#"open location "https://google.com""#;
        assert_eq!(
            AppleScriptShim::extract_open_location(script),
            Some("https://google.com".to_string())
        );
    }

    #[test]
    fn test_extract_open_location_file() {
        let script = r#"open "/tmp/test.txt""#;
        assert_eq!(
            AppleScriptShim::extract_open_location(script),
            Some("/tmp/test.txt".to_string())
        );
    }

    #[test]
    fn test_extract_open_finder() {
        let script = r#"tell application "Finder" to open "/Users/test/Documents""#;
        assert_eq!(
            AppleScriptShim::extract_open_location(script),
            Some("/Users/test/Documents".to_string())
        );
    }

    #[test]
    fn test_extract_set_clipboard() {
        let script = r#"set the clipboard to "hello world""#;
        assert_eq!(
            AppleScriptShim::extract_set_clipboard(script),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_is_get_clipboard() {
        assert!(AppleScriptShim::is_get_clipboard("get the clipboard"));
        assert!(AppleScriptShim::is_get_clipboard("the clipboard"));
        assert!(!AppleScriptShim::is_get_clipboard("set the clipboard"));
    }

    #[test]
    fn test_detect_display_server() {
        // This test will pass regardless of what display server is running
        let display = AppleScriptShim::detect_display_server();
        assert!(matches!(
            display,
            DisplayServer::X11 | DisplayServer::Wayland | DisplayServer::Unknown
        ));
    }

    // Integration test: end-to-end shell script execution
    #[test]
    fn test_run_apple_script_shell() {
        let script = r#"do shell script "echo 'test output'""#;
        let result = AppleScriptShim::run_apple_script(script);
        assert!(result.success);
        assert!(result.output.is_some());
    }

    // Integration test: notification fallback when pattern not supported
    #[test]
    fn test_run_apple_script_unsupported() {
        let script = r#"tell application "SystemUIServer" to do something complex"#;
        let result = AppleScriptShim::run_apple_script(script);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not supported"));
    }

    // ========== NEW PRIORITY 2 TESTS (GUI AUTOMATION) ==========

    #[test]
    fn test_extract_keystroke_simple() {
        let script = r#"keystroke "hello""#;
        assert_eq!(
            AppleScriptShim::extract_keystroke(script),
            Some(("hello".to_string(), Vec::new()))
        );
    }

    #[test]
    fn test_extract_keystroke_with_modifiers() {
        let script = r#"keystroke "c" using {command down}"#;
        let result = AppleScriptShim::extract_keystroke(script);
        assert!(result.is_some());
        let (text, modifiers) = result.unwrap();
        assert_eq!(text, "c");
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0], Modifier::Command);
    }

    #[test]
    fn test_extract_keystroke_multiple_modifiers() {
        let script = r#"keystroke "v" using {command down, shift down}"#;
        let result = AppleScriptShim::extract_keystroke(script);
        assert!(result.is_some());
        let (text, modifiers) = result.unwrap();
        assert_eq!(text, "v");
        assert!(modifiers.contains(&Modifier::Command));
        assert!(modifiers.contains(&Modifier::Shift));
    }

    #[test]
    fn test_extract_keycode() {
        let script = r#"key code 36"#;
        assert_eq!(
            AppleScriptShim::extract_keycode(script),
            Some((36, Vec::new()))
        );
    }

    #[test]
    fn test_extract_keycode_with_modifiers() {
        let script = r#"key code 36 using {command down}"#;
        let result = AppleScriptShim::extract_keycode(script);
        assert!(result.is_some());
        let (code, modifiers) = result.unwrap();
        assert_eq!(code, 36);
        assert_eq!(modifiers, vec![Modifier::Command]);
    }

    #[test]
    fn test_parse_modifiers() {
        let mods = AppleScriptShim::parse_modifiers("command down, shift down");
        assert_eq!(mods.len(), 2);
        assert!(mods.contains(&Modifier::Command));
        assert!(mods.contains(&Modifier::Shift));
    }

    #[test]
    fn test_macos_keycode_to_linux() {
        assert_eq!(AppleScriptShim::macos_keycode_to_linux(36), "Return");
        assert_eq!(AppleScriptShim::macos_keycode_to_linux(51), "BackSpace");
        assert_eq!(AppleScriptShim::macos_keycode_to_linux(53), "Escape");
    }

    #[test]
    fn test_modifiers_to_x11_keys() {
        let mods = vec![Modifier::Command, Modifier::Shift];
        assert_eq!(AppleScriptShim::modifiers_to_x11_keys(&mods), "super+shift");
    }
}
