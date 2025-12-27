use serde::{Deserialize, Serialize};
use std::process::Command;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

/// Window geometry (position and size)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Monitor/display information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Window snap positions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SnapPosition {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeftQuarter,
    TopRightQuarter,
    BottomLeftQuarter,
    BottomRightQuarter,
    Center,
    Maximize,
    AlmostMaximize,
}

/// Get X11 connection
fn get_x11_connection() -> Result<(RustConnection, usize), String> {
    RustConnection::connect(None).map_err(|e| format!("Failed to connect to X11: {}", e))
}

/// Get the currently active window
fn get_active_window() -> Result<Window, String> {
    let (conn, screen_num) = get_x11_connection()?;
    let screen = &conn.setup().roots[screen_num];

    // Get _NET_ACTIVE_WINDOW atom
    let atom_reply = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .map_err(|e| format!("Failed to intern atom: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get atom reply: {}", e))?;

    // Get the active window property
    let reply = conn
        .get_property(
            false, // Don't delete
            screen.root,
            atom_reply.atom,
            AtomEnum::WINDOW,
            0,
            1,
        )
        .map_err(|e| format!("Failed to get property: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get reply: {}", e))?;

    // Parse window ID from reply value
    if reply.value.len() >= 4 {
        let window_id = u32::from_ne_bytes([
            reply.value[0],
            reply.value[1],
            reply.value[2],
            reply.value[3],
        ]);
        tracing::info!("Active window ID: {}", window_id);
        Ok(window_id)
    } else {
        Err("No active window found".to_string())
    }
}

/// Get window geometry (position and size)
fn get_window_geometry(window: Window) -> Result<WindowGeometry, String> {
    let (conn, screen_num) = get_x11_connection()?;
    let screen = &conn.setup().roots[screen_num];

    // Get window geometry
    let geometry = conn
        .get_geometry(window)
        .map_err(|e| format!("Failed to get geometry: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get geometry reply: {}", e))?;

    // Translate coordinates to root window coordinates
    let translate = conn
        .translate_coordinates(window, screen.root, 0, 0)
        .map_err(|e| format!("Failed to translate coordinates: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get translate reply: {}", e))?;

    let geom = WindowGeometry {
        x: translate.dst_x as i32,
        y: translate.dst_y as i32,
        width: geometry.width as u32,
        height: geometry.height as u32,
    };

    tracing::info!("Window geometry: {:?}", geom);
    Ok(geom)
}

/// Get all  monitors
pub fn get_monitors() -> Result<Vec<Monitor>, String> {
    tracing::info!("Getting monitors via xrandr");

    let output = Command::new("xrandr")
        .arg("--current")
        .output()
        .map_err(|e| format!("Failed to run xrandr: {}", e))?;

    if !output.status.success() {
        return Err("xrandr command failed".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();

    for line in output_str.lines() {
        if line.contains(" connected") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let name = parts[0].to_string();
            let is_primary = parts.contains(&"primary");

            // Find geometry string (format: "1920x1080+1920+0")
            let geom_str = parts
                .iter()
                .find(|s| s.contains('x') && s.contains('+'))
                .ok_or("Invalid monitor geometry")?;

            // Parse geometry: "1920x1080+1920+0"
            let (size, pos) = geom_str.split_once('+').ok_or("Invalid geometry format")?;
            let (width_str, height_str) = size.split_once('x').ok_or("Invalid size format")?;

            // Handle position with two + separators
            let pos_parts: Vec<&str> = pos.splitn(2, '+').collect();
            let x: i32 = pos_parts[0].parse().map_err(|_| "Invalid x coordinate")?;
            let y: i32 = pos_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

            let width: u32 = width_str.parse().map_err(|_| "Invalid width")?;
            let height: u32 = height_str.parse().map_err(|_| "Invalid height")?;

            monitors.push(Monitor {
                name,
                x,
                y,
                width,
                height,
                is_primary,
            });
        }
    }

    tracing::info!("Found {} monitors", monitors.len());
    Ok(monitors)
}

/// Get the monitor that contains the given window
fn get_window_monitor(window: Window) -> Result<Monitor, String> {
    let geom = get_window_geometry(window)?;
    let monitors = get_monitors()?;

    // Find monitor that contains the window center
    let center_x = geom.x + (geom.width / 2) as i32;
    let center_y = geom.y + (geom.height / 2) as i32;

    for monitor in &monitors {
        if center_x >= monitor.x
            && center_x < monitor.x + monitor.width as i32
            && center_y >= monitor.y
            && center_y < monitor.y + monitor.height as i32
        {
            tracing::info!("Window is on monitor: {}", monitor.name);
            return Ok(monitor.clone());
        }
    }

    // Fallback to primary monitor
    monitors
        .iter()
        .find(|m| m.is_primary)
        .cloned()
        .ok_or("No monitor found for window".to_string())
}

/// Move and resize a window
fn move_resize_window(
    window: Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let (conn, _) = get_x11_connection()?;

    tracing::info!(
        "Moving/resizing window to x={}, y={}, w={}, h={}",
        x,
        y,
        width,
        height
    );

    let values = ConfigureWindowAux::new()
        .x(x)
        .y(y)
        .width(width)
        .height(height);

    conn.configure_window(window, &values)
        .map_err(|e| format!("Failed to configure window: {}", e))?;

    conn.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    Ok(())
}

/// Snap the active window to a position
#[tauri::command]
pub async fn snap_active_window(position: SnapPosition) -> Result<(), String> {
    tracing::info!("Snapping window to: {:?}", position);

    let window = get_active_window()?;
    let monitor = get_window_monitor(window)?;

    // Account for Cinnamon panel (usually bottom, ~30px)
    const PANEL_HEIGHT: u32 = 30;
    const ALMOST_MAX_PADDING: u32 = 20;

    let usable_height = monitor.height.saturating_sub(PANEL_HEIGHT);

    let (x, y, width, height) = match position {
        SnapPosition::LeftHalf => (monitor.x, monitor.y, monitor.width / 2, usable_height),
        SnapPosition::RightHalf => (
            monitor.x + (monitor.width / 2) as i32,
            monitor.y,
            monitor.width / 2,
            usable_height,
        ),
        SnapPosition::TopHalf => (monitor.x, monitor.y, monitor.width, usable_height / 2),
        SnapPosition::BottomHalf => (
            monitor.x,
            monitor.y + (usable_height / 2) as i32,
            monitor.width,
            usable_height / 2,
        ),
        SnapPosition::TopLeftQuarter => {
            (monitor.x, monitor.y, monitor.width / 2, usable_height / 2)
        }
        SnapPosition::TopRightQuarter => (
            monitor.x + (monitor.width / 2) as i32,
            monitor.y,
            monitor.width / 2,
            usable_height / 2,
        ),
        SnapPosition::BottomLeftQuarter => (
            monitor.x,
            monitor.y + (usable_height / 2) as i32,
            monitor.width / 2,
            usable_height / 2,
        ),
        SnapPosition::BottomRightQuarter => (
            monitor.x + (monitor.width / 2) as i32,
            monitor.y + (usable_height / 2) as i32,
            monitor.width / 2,
            usable_height / 2,
        ),
        SnapPosition::Center => {
            let new_width = (monitor.width as f32 * 0.7) as u32;
            let new_height = (usable_height as f32 * 0.7) as u32;
            (
                monitor.x + ((monitor.width - new_width) / 2) as i32,
                monitor.y + ((usable_height - new_height) / 2) as i32,
                new_width,
                new_height,
            )
        }
        SnapPosition::Maximize => (monitor.x, monitor.y, monitor.width, usable_height),
        SnapPosition::AlmostMaximize => (
            monitor.x + ALMOST_MAX_PADDING as i32,
            monitor.y + ALMOST_MAX_PADDING as i32,
            monitor.width - (ALMOST_MAX_PADDING * 2),
            usable_height - (ALMOST_MAX_PADDING * 2),
        ),
    };

    move_resize_window(window, x, y, width, height)?;
    Ok(())
}

/// Get available monitors
#[tauri::command]
pub async fn get_available_monitors() -> Result<Vec<Monitor>, String> {
    get_monitors()
}

/// Move active window to a specific monitor
#[tauri::command]
pub async fn move_window_to_monitor(monitor_index: usize) -> Result<(), String> {
    tracing::info!("Moving window to monitor index: {}", monitor_index);

    let window = get_active_window()?;
    let current_geom = get_window_geometry(window)?;
    let monitors = get_monitors()?;

    if monitor_index >= monitors.len() {
        return Err(format!("Monitor index {} out of range", monitor_index));
    }

    let target_monitor = &monitors[monitor_index];

    // Center window on target monitor, keeping current size
    let x = target_monitor.x + ((target_monitor.width - current_geom.width) / 2) as i32;
    let y = target_monitor.y + ((target_monitor.height - current_geom.height) / 2) as i32;

    move_resize_window(window, x, y, current_geom.width, current_geom.height)?;
    Ok(())
}
