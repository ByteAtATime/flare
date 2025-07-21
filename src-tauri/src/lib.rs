mod ai;
mod app;
mod browser_extension;
mod cache;
mod clipboard;
pub mod clipboard_history;
mod desktop;
mod error;
mod extensions;
mod file_search;
mod filesystem;
mod frecency;
mod oauth;
mod quicklinks;
mod snippets;
mod soulver;
mod store;
mod system;

use crate::snippets::input_manager::{EvdevInputManager, InputManager, RdevInputManager};
use crate::{app::App, cache::AppCache};
use ai::AiUsageManager;
use browser_extension::WsState;
use frecency::FrecencyManager;
use quicklinks::QuicklinkManager;
use selection::get_text;
use snippets::engine::ExpansionEngine;
use snippets::manager::SnippetManager;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[tauri::command]
fn get_installed_apps() -> Vec<App> {
    match AppCache::get_apps() {
        Ok(apps) => apps,
        Err(e) => {
            eprintln!("Failed to get apps: {:?}", e);
            Vec::new()
        }
    }
}

#[tauri::command]
fn launch_app(exec: String) -> Result<(), String> {
    let exec_parts: Vec<&str> = exec.split_whitespace().collect();
    if exec_parts.is_empty() {
        return Err("Empty exec command".to_string());
    }

    let mut command = Command::new(exec_parts[0]);
    for arg in &exec_parts[1..] {
        if !arg.starts_with('%') {
            command.arg(arg);
        }
    }

    command
        .spawn()
        .map_err(|e| format!("Failed to launch app: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_selected_text() -> String {
    get_text()
}

#[tauri::command]
async fn show_hud(app: tauri::AppHandle, title: String) -> Result<(), String> {
    let hud_window = match app.get_webview_window("hud") {
        Some(window) => window,
        None => {
            tauri::WebviewWindowBuilder::new(&app, "hud", tauri::WebviewUrl::App("/hud".into()))
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .center()
                .min_inner_size(300.0, 80.0)
                .max_inner_size(300.0, 80.0)
                .inner_size(300.0, 80.0)
                .build()
                .map_err(|e| e.to_string())?
        }
    };

    let window_clone = hud_window.clone();
    window_clone.show().map_err(|e| e.to_string())?;
    window_clone
        .emit("hud-message", &title)
        .map_err(|e| e.to_string())?;
    window_clone
        .set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
    window_clone.set_focus().map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let _ = window_clone.hide();
    });

    Ok(())
}

#[tauri::command]
fn record_usage(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    app.state::<FrecencyManager>()
        .record_usage(item_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_frecency_data(app: tauri::AppHandle) -> Result<Vec<frecency::FrecencyData>, String> {
    app.state::<FrecencyManager>()
        .get_frecency_data()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_frecency_entry(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    app.state::<FrecencyManager>()
        .delete_frecency_entry(item_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_item(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    app.state::<FrecencyManager>()
        .hide_item(item_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_hidden_item_ids(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    app.state::<FrecencyManager>()
        .get_hidden_item_ids()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_discovered_plugins(app: tauri::AppHandle) -> Result<Vec<extensions::PluginInfo>, String> {
    extensions::discover_plugins(&app)
}

fn setup_background_refresh() {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(60));
        loop {
            AppCache::refresh_background();
            thread::sleep(Duration::from_secs(300));
        }
    });
}

fn setup_global_shortcut(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let spotlight_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    let handle = app.handle().clone();

    println!("Spotlight shortcut: {:?}", spotlight_shortcut);

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                println!("Shortcut: {:?}, Event: {:?}", shortcut, event);
                if shortcut == &spotlight_shortcut && event.state() == ShortcutState::Pressed {
                    let spotlight_window = handle.get_webview_window("main").unwrap();
                    println!("Spotlight window: {:?}", spotlight_window);
                    if spotlight_window.is_visible().unwrap_or(false) {
                        spotlight_window.hide().unwrap();
                    } else {
                        spotlight_window.show().unwrap();
                        spotlight_window.set_focus().unwrap();
                    }
                }
            })
            .build(),
    )?;

    if !app.global_shortcut().is_registered(spotlight_shortcut) {
        app.global_shortcut().register(spotlight_shortcut)?;
    }
    Ok(())
}

fn setup_input_listener(app: &tauri::AppHandle) {
    let snippet_manager = app.state::<SnippetManager>().inner().clone();
    let snippet_manager_arc = Arc::new(snippet_manager);

    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    let input_manager_result: Result<Arc<dyn InputManager>, anyhow::Error> = if is_wayland {
        println!("[Snippets] Wayland detected, using evdev for snippet expansion.");
        EvdevInputManager::new().map(|m| Arc::new(m) as Arc<dyn InputManager>)
    } else {
        println!("[Snippets] X11 or unknown session, using rdev for snippet expansion.");
        RdevInputManager::new().map(|m| Arc::new(m) as Arc<dyn InputManager>)
    };

    match input_manager_result {
        Ok(input_manager) => {
            app.manage(input_manager.clone());

            let engine = ExpansionEngine::new(snippet_manager_arc, input_manager);
            thread::spawn(move || {
                if let Err(e) = engine.start_listening() {
                    eprintln!("[ExpansionEngine] Failed to start: {}", e);
                }
            });
        }
        Err(e) => {
            eprintln!(
                "[Snippets] Failed to initialize input manager: {}. Snippet expansion will be disabled.",
                e
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .manage(WsState::default())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if args.len() > 1 && args[1].starts_with("raycast://") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("deep-link", args[1].to_string());
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                return;
            }

            if let Some(window) = app.get_webview_window("main") {
                if let Ok(true) = window.is_visible() {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            launch_app,
            get_selected_text,
            show_hud,
            get_discovered_plugins,
            filesystem::get_selected_finder_items,
            extensions::install_extension,
            browser_extension::browser_extension_check_connection,
            browser_extension::browser_extension_request,
            clipboard::clipboard_read_text,
            clipboard::clipboard_read,
            clipboard::clipboard_copy,
            clipboard::clipboard_paste,
            clipboard::clipboard_clear,
            oauth::oauth_set_tokens,
            oauth::oauth_get_tokens,
            oauth::oauth_remove_tokens,
            clipboard_history::history_get_items,
            clipboard_history::history_get_item_content,
            clipboard_history::history_delete_item,
            clipboard_history::history_toggle_pin,
            clipboard_history::history_clear_all,
            clipboard_history::history_item_was_copied,
            quicklinks::create_quicklink,
            quicklinks::list_quicklinks,
            quicklinks::update_quicklink,
            quicklinks::delete_quicklink,
            quicklinks::execute_quicklink,
            system::get_applications,
            system::get_default_application,
            system::get_frontmost_application,
            system::show_in_finder,
            system::trash,
            record_usage,
            get_frecency_data,
            delete_frecency_entry,
            hide_item,
            get_hidden_item_ids,
            snippets::create_snippet,
            snippets::list_snippets,
            snippets::update_snippet,
            snippets::delete_snippet,
            snippets::import_snippets,
            snippets::paste_snippet_content,
            snippets::snippet_was_used,
            file_search::search_files,
            ai::set_ai_api_key,
            ai::is_ai_api_key_set,
            ai::clear_ai_api_key,
            ai::ai_ask_stream,
            ai::get_ai_usage_history,
            ai::get_ai_settings,
            ai::set_ai_settings,
            ai::ai_can_access,
            soulver::calculate_soulver
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(browser_extension::run_server(app_handle));

            clipboard_history::init(app.handle().clone());
            file_search::init(app.handle().clone());

            app.manage(QuicklinkManager::new(app.handle())?);
            app.manage(FrecencyManager::new(app.handle())?);
            app.manage(SnippetManager::new(app.handle())?);
            app.manage(AiUsageManager::new(app.handle())?);

            setup_background_refresh();
            setup_global_shortcut(app)?;
            setup_input_listener(app.handle());

            let soulver_core_path = app
                .path()
                .resource_dir()
                .unwrap()
                .join("SoulverWrapper/Vendor/SoulverCore-linux");

            soulver::initialize(soulver_core_path.to_str().unwrap());

            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap();

    app.run(|app, event| {
        if let tauri::RunEvent::WindowEvent { label, event, .. } = event {
            if label == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    tauri::WindowEvent::Focused(false) => {
                        if let Some(window) = app.get_webview_window("main") {
                            if !cfg!(debug_assertions) {
                                let _ = window.hide();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}
