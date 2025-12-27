mod encryption;
pub mod manager;
mod monitor;
pub mod types;

pub use manager::init;
use manager::MANAGER;
use types::ClipboardItem;

#[tauri::command]
pub fn history_get_items(
    filter: String,
    search_term: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<Vec<ClipboardItem>, String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager
            .get_items(filter, search_term, limit, offset)
            .map_err(|e| e.to_string())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn history_get_item_content(id: i64) -> Result<String, String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager.get_item_content(id).map_err(|e| e.to_string())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn history_item_was_copied(id: i64) -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager.item_was_copied(id).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn history_delete_item(id: i64) -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager.delete_item(id).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn history_toggle_pin(id: i64) -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager.toggle_pin(id).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}

#[tauri::command]
pub fn history_clear_all() -> Result<(), String> {
    if let Some(manager) = MANAGER
        .lock()
        .expect("clipboard manager mutex poisoned")
        .as_ref()
    {
        manager.clear_all().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Clipboard history manager not initialized".to_string())
    }
}
