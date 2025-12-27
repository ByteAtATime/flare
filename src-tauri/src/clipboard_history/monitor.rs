use super::{
    manager::MANAGER,
    types::{ContentType, COLOR_REGEX, URL_REGEX},
};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tauri::AppHandle;

pub fn start_monitoring(_app_handle: AppHandle) {
    std::thread::spawn(move || {
        let mut last_text_hash = String::new();
        let mut last_image_hash = String::new();
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize clipboard monitor");
                return;
            }
        };

        loop {
            if super::manager::INTERNAL_CLIPBOARD_CHANGE.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            if let Ok(text) = clipboard.get_text() {
                let text = text.trim();
                if !text.is_empty() {
                    let current_hash = hex::encode(Sha256::digest(text.as_bytes()));
                    if current_hash != last_text_hash {
                        let (content_type, content_value) = if COLOR_REGEX.is_match(text) {
                            (ContentType::Color, text.to_string())
                        } else if URL_REGEX.is_match(text) {
                            (ContentType::Link, text.to_string())
                        } else {
                            (ContentType::Text, text.to_string())
                        };

                        if let Ok(guard) = MANAGER.lock() {
                            if let Some(manager) = guard.as_ref() {
                                if let Err(e) = manager.add_item(
                                    current_hash.clone(),
                                    content_type,
                                    content_value,
                                    None,
                                ) {
                                    tracing::error!(error = ?e, "Error adding clipboard text item");
                                }
                            }
                        }
                        last_text_hash = current_hash;
                        last_image_hash.clear();
                    }
                }
            }

            if let Ok(image_data) = clipboard.get_image() {
                let current_hash = hex::encode(Sha256::digest(&image_data.bytes));
                if current_hash != last_image_hash {
                    if let Ok(guard) = MANAGER.lock() {
                        if let Some(manager) = guard.as_ref() {
                            let image_path =
                                manager.image_dir.join(format!("{}.png", current_hash));
                            match image::save_buffer(
                                &image_path,
                                &image_data.bytes,
                                image_data.width as u32,
                                image_data.height as u32,
                                image::ColorType::Rgba8,
                            ) {
                                Ok(_) => {
                                    let content_value = image_path.to_string_lossy().to_string();
                                    if let Err(e) = manager.add_item(
                                        current_hash.clone(),
                                        ContentType::Image,
                                        content_value,
                                        None,
                                    ) {
                                        tracing::error!(error = ?e, "Error adding clipboard image item");
                                    }
                                }
                                Err(e) => tracing::error!(error = ?e, "Failed to save image"),
                            }
                        }
                    }
                    last_image_hash = current_hash;
                    last_text_hash.clear();
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
