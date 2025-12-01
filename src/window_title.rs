// like literally all of this code is copied directly from https://github.com/savedra1/clipse/blob/main/utils/window.go

use std::env;
use std::process::Command;

pub fn get_active_window_title() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        return try_macos();
    }

    #[cfg(target_os = "linux")]
    {
        if env::var("WAYLAND_DISPLAY").is_ok() {
            if let Some(title) = try_hyprctl() {
                return Some(title);
            }
            if let Some(title) = try_wlrctl() {
                return Some(title);
            }
        }

        if let Some(title) = try_xdotool() {
            return Some(title);
        }
        if let Some(title) = try_xprop() {
            return Some(title);
        }

        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

fn exec(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            } else {
                None
            }
        })
}

#[cfg(target_os = "macos")]
fn try_macos() -> Option<String> {
    exec(
        "osascript",
        &[
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ],
    )
}

#[cfg(target_os = "linux")]
fn try_hyprctl() -> Option<String> {
    let output = exec("hyprctl", &["activewindow", "-j"])?;
    let json: serde_json::Value = serde_json::from_str(&output).ok()?;

    if let Some(class) = json.get("class").and_then(|v| v.as_str()) {
        if !class.is_empty() {
            return Some(class.to_string());
        }
    }
    json.get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(target_os = "linux")]
fn try_wlrctl() -> Option<String> {
    let output = exec("wlrctl", &["toplevel", "list", "state:focused"])?;

    let first_line = output.lines().next()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let app_name = parts[0].strip_suffix(':').unwrap_or(parts[0]);
    Some(app_name.to_string())
}

#[cfg(target_os = "linux")]
fn try_xdotool() -> Option<String> {
    exec("xdotool", &["getactivewindow", "getwindowname"])
}

#[cfg(target_os = "linux")]
fn try_xprop() -> Option<String> {
    let root_out = exec("xprop", &["-root", "_NET_ACTIVE_WINDOW"])?;

    let id_marker = "# ";
    let idx = root_out.find(id_marker)?;
    let id_part = &root_out[idx + id_marker.len()..];
    let window_id = id_part.split_whitespace().next()?;

    let name_out = exec("xprop", &["-id", window_id, "WM_NAME"])?;

    let start_quote = name_out.find('"')?;
    let end_quote = name_out.rfind('"')?;

    if start_quote < end_quote {
        Some(name_out[start_quote + 1..end_quote].to_string())
    } else {
        None
    }
}
