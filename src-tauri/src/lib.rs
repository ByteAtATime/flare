mod ai;
mod ai_presets;
mod aliases;
mod app;
mod floating_notes;
mod script_commands;
mod auto_start;
mod browser_extension;
mod cache;
mod cli_substitutes;
mod clipboard;
pub mod clipboard_history;
mod desktop;
pub mod dmenu;
mod downloads;
mod error;
mod extension_shims;
mod extensions;
mod file_search;
mod filesystem;
mod frecency;
mod hotkey_manager;
mod integrations;
mod oauth;
mod quick_toggles;
mod quicklinks;
mod settings;
mod snippets;
mod soulver;
mod store;
mod system;
mod system_commands;
mod system_monitors;
mod window_management;

use crate::snippets::input_manager::{EvdevInputManager, InputManager, RdevInputManager};
use crate::{app::App, cache::AppCache};
use ai::AiUsageManager;
use ai_presets::AiPresetManager;
use aliases::AliasManager;
use browser_extension::WsState;
use floating_notes::FloatingNotesManager;
use frecency::FrecencyManager;
use script_commands::ScriptCommandManager;
use quicklinks::QuicklinkManager;
use selection::get_text;
use snippets::engine::ExpansionEngine;
use snippets::manager::SnippetManager;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder};

use dmenu::DmenuSession;

// Global state for dmenu session (only used in dmenu mode)
static DMENU_SESSION: Mutex<Option<DmenuSession>> = Mutex::new(None);

#[tauri::command]
fn get_installed_apps(app: tauri::AppHandle) -> Vec<App> {
    match AppCache::get_apps(&app) {
        Ok(apps) => apps,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to get installed apps");
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

fn setup_background_refresh(app: tauri::AppHandle) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(60));
        loop {
            AppCache::refresh_background(app.clone());
            thread::sleep(Duration::from_secs(300));
        }
    });
}

fn setup_global_shortcut(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let spotlight_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);

    // Register the shortcut handler
    tracing::info!("Registering global shortcut: Ctrl+Alt+Space");
    app.global_shortcut()
        .on_shortcut(spotlight_shortcut, move |app, shortcut, event| {
            tracing::debug!(
                shortcut = ?shortcut,
                state = ?event.state(),
                "Hotkey event received"
            );

            if event.state() == ShortcutState::Pressed {
                tracing::debug!("Processing hotkey PRESSED event");

                if let Some(window) = app.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(true) => {
                            tracing::debug!("Window visible, hiding");
                            let _ = window.hide();
                        }
                        Ok(false) => {
                            tracing::debug!("Window hidden, showing");
                            let _ = window.show();
                            // Ensure window is on top (Linux WMs sometimes ignore config setting)
                            let _ = window.set_always_on_top(true);
                            // Request focus immediately
                            let _ = window.set_focus();
                            // Use xdotool to force focus via mouse click (bypasses WM focus-stealing prevention)
                            let window_clone = window.clone();
                            tauri::async_runtime::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                // Get window position and size to click in the center (on the input)
                                if let (Ok(pos), Ok(size)) =
                                    (window_clone.outer_position(), window_clone.outer_size())
                                {
                                    // Click near the top center where the search input is
                                    let click_x = pos.x + (size.width as i32 / 2);
                                    let click_y = pos.y + 40; // Near top for the input
                                    let _ = std::process::Command::new("xdotool")
                                        .args([
                                            "mousemove",
                                            "--sync",
                                            &click_x.to_string(),
                                            &click_y.to_string(),
                                            "click",
                                            "1",
                                        ])
                                        .stderr(std::process::Stdio::null())
                                        .stdout(std::process::Stdio::null())
                                        .spawn();
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to check window visibility");
                        }
                    }
                } else {
                    tracing::error!("Main window not found");
                }
            } else {
                tracing::trace!("Ignoring hotkey RELEASED event");
            }
        })?;

    app.global_shortcut().register(spotlight_shortcut)?;
    tracing::info!("Global shortcut registered successfully");

    Ok(())
}

fn setup_input_listener(app: &tauri::AppHandle) {
    let snippet_manager = app.state::<SnippetManager>().inner().clone();
    let snippet_manager_arc = Arc::new(snippet_manager);

    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    let input_manager_result: Result<Arc<dyn InputManager>, anyhow::Error> = if is_wayland {
        tracing::info!("Wayland detected, using evdev for snippet expansion");
        EvdevInputManager::new().map(|m| Arc::new(m) as Arc<dyn InputManager>)
    } else {
        tracing::info!("X11 or unknown session, using rdev for snippet expansion");
        RdevInputManager::new().map(|m| Arc::new(m) as Arc<dyn InputManager>)
    };

    match input_manager_result {
        Ok(input_manager) => {
            app.manage(input_manager.clone());

            let engine = ExpansionEngine::new(snippet_manager_arc, input_manager);
            thread::spawn(move || {
                if let Err(e) = engine.start_listening() {
                    tracing::error!(error = %e, "Expansion engine failed to start");
                }
            });
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                "Failed to initialize input manager, snippet expansion will be disabled"
            );
        }
    }
}

// Extension shim commands
#[tauri::command]
fn shim_translate_path(path: String) -> String {
    extension_shims::PathShim::translate_path(&path)
}

#[tauri::command]
fn shim_run_applescript(script: String) -> extension_shims::ShimResult {
    extension_shims::AppleScriptShim::run_apple_script(&script)
}

#[tauri::command]
fn shim_get_system_info() -> std::collections::HashMap<String, String> {
    extension_shims::SystemShim::get_system_info()
}

// System monitor commands
#[tauri::command]
fn monitor_get_cpu() -> system_monitors::CpuInfo {
    system_monitors::get_cpu_info()
}

#[tauri::command]
fn monitor_get_memory() -> system_monitors::MemoryInfo {
    system_monitors::get_memory_info()
}

#[tauri::command]
fn monitor_get_disks() -> Vec<system_monitors::DiskInfo> {
    system_monitors::get_disk_info()
}

#[tauri::command]
fn monitor_get_network() -> Vec<system_monitors::NetworkInfo> {
    system_monitors::get_network_info()
}

#[tauri::command]
fn monitor_get_battery() -> Option<system_monitors::BatteryInfo> {
    system_monitors::get_battery_info()
}

// Quick toggle commands
#[tauri::command]
async fn toggle_wifi(enable: bool) -> Result<(), String> {
    quick_toggles::toggle_wifi(enable).await
}

#[tauri::command]
async fn get_wifi_state() -> Result<bool, String> {
    quick_toggles::get_wifi_state().await
}

#[tauri::command]
async fn toggle_bluetooth(enable: bool) -> Result<(), String> {
    quick_toggles::toggle_bluetooth(enable).await
}

#[tauri::command]
async fn get_bluetooth_state() -> Result<bool, String> {
    quick_toggles::get_bluetooth_state().await
}

#[tauri::command]
async fn toggle_dark_mode(enable: bool) -> Result<(), String> {
    quick_toggles::toggle_dark_mode(enable).await
}

#[tauri::command]
async fn get_dark_mode_state() -> Result<bool, String> {
    quick_toggles::get_dark_mode_state().await
}

#[tauri::command]
fn set_brightness(percentage: u32) -> Result<(), String> {
    quick_toggles::set_brightness(percentage)
}

#[tauri::command]
fn get_brightness() -> Result<u32, String> {
    quick_toggles::get_brightness()
}

// GitHub integration commands
#[tauri::command]
async fn github_start_auth() -> Result<integrations::github::DeviceCodeResponse, String> {
    integrations::github::start_device_flow().await
}

#[tauri::command]
async fn github_poll_auth(device_code: String) -> Result<Option<String>, String> {
    integrations::github::poll_for_token(&device_code).await
}

#[tauri::command]
fn github_store_token(token: String) -> Result<(), String> {
    integrations::github::store_token(&token)
}

#[tauri::command]
fn github_is_authenticated() -> Result<bool, String> {
    Ok(integrations::github::get_token()?.is_some())
}

#[tauri::command]
fn github_logout() -> Result<(), String> {
    integrations::github::delete_token()
}

#[tauri::command]
async fn github_get_current_user() -> Result<integrations::github::User, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.get_current_user().await
}

// GitHub Issues commands
#[tauri::command]
async fn github_list_issues(
    owner: String,
    repo: String,
    state: Option<String>,
) -> Result<Vec<integrations::github::Issue>, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.list_issues(&owner, &repo, state.as_deref()).await
}

#[tauri::command]
async fn github_get_issue(
    owner: String,
    repo: String,
    number: u64,
) -> Result<integrations::github::Issue, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.get_issue(&owner, &repo, number).await
}

#[tauri::command]
async fn github_create_issue(
    owner: String,
    repo: String,
    title: String,
    body: Option<String>,
    labels: Option<Vec<String>>,
    assignees: Option<Vec<String>>,
) -> Result<integrations::github::Issue, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client
        .create_issue(&owner, &repo, title, body, labels, assignees)
        .await
}

#[tauri::command]
async fn github_update_issue(
    owner: String,
    repo: String,
    number: u64,
    title: Option<String>,
    body: Option<String>,
    state: Option<String>,
    labels: Option<Vec<String>>,
    assignees: Option<Vec<String>>,
) -> Result<integrations::github::Issue, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client
        .update_issue(
            &owner,
            &repo,
            number,
            title,
            body,
            state.as_deref(),
            labels,
            assignees,
        )
        .await
}

#[tauri::command]
async fn github_close_issue(
    owner: String,
    repo: String,
    number: u64,
) -> Result<integrations::github::Issue, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.close_issue(&owner, &repo, number).await
}

#[tauri::command]
async fn github_list_my_issues(
    state: Option<String>,
) -> Result<Vec<integrations::github::Issue>, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.list_my_issues(state.as_deref()).await
}

// GitHub Search commands
#[tauri::command]
async fn github_search_issues(
    query: String,
) -> Result<integrations::github::SearchResult<integrations::github::Issue>, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.search_issues(&query).await
}

#[tauri::command]
async fn github_search_repos(
    query: String,
) -> Result<integrations::github::SearchResult<integrations::github::Repository>, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.search_repos(&query).await
}

// GitHub Repository commands
#[tauri::command]
async fn github_list_repos() -> Result<Vec<integrations::github::Repository>, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.list_user_repos().await
}

#[tauri::command]
async fn github_get_repo(
    owner: String,
    repo: String,
) -> Result<integrations::github::Repository, String> {
    let client = integrations::github::GitHubClient::from_stored_token()?;
    client.get_repo(&owner, &repo).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

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
            soulver::calculate_soulver,
            shim_translate_path,
            shim_run_applescript,
            shim_get_system_info,
            monitor_get_cpu,
            monitor_get_memory,
            monitor_get_disks,
            monitor_get_network,
            monitor_get_battery,
            toggle_wifi,
            get_wifi_state,
            toggle_bluetooth,
            get_bluetooth_state,
            toggle_dark_mode,
            get_dark_mode_state,
            set_brightness,
            get_brightness,
            github_start_auth,
            github_poll_auth,
            github_store_token,
            github_is_authenticated,
            github_logout,
            github_get_current_user,
            github_list_issues,
            github_get_issue,
            github_create_issue,
            github_update_issue,
            github_close_issue,
            github_list_my_issues,
            github_search_issues,
            github_search_repos,
            github_list_repos,
            github_get_repo,
            ai::get_ollama_models,
            ai::create_conversation,
            ai::list_conversations,
            ai::get_conversation,
            ai::update_conversation,
            ai::delete_conversation,
            ai_presets::get_ai_presets,
            ai_presets::create_ai_preset,
            ai_presets::update_ai_preset,
            ai_presets::delete_ai_preset,
            aliases::get_aliases,
            aliases::set_alias,
            aliases::remove_alias,
            script_commands::get_script_commands,
            script_commands::run_script_command,
            script_commands::open_scripts_folder,
            floating_notes::get_floating_note,
            floating_notes::save_floating_note,
            floating_notes::toggle_floating_notes_window,
            system_commands::execute_power_command,
            system_commands::set_volume,
            system_commands::volume_up,
            system_commands::volume_down,
            system_commands::toggle_mute,
            system_commands::get_volume,
            system_commands::empty_trash,
            system_commands::eject_drive,
            window_management::snap_active_window,
            window_management::get_available_monitors,
            window_management::move_window_to_monitor,
            hotkey_manager::get_hotkey_config,
            hotkey_manager::set_command_hotkey,
            hotkey_manager::remove_command_hotkey,
            hotkey_manager::check_hotkey_conflict,
            hotkey_manager::reset_hotkeys_to_defaults,
            downloads::downloads_get_items,
            downloads::downloads_open_file,
            downloads::downloads_show_in_folder,
            downloads::downloads_delete_item,
            downloads::downloads_delete_file,
            downloads::downloads_clear_history,
            downloads::downloads_get_latest,
            downloads::downloads_copy_latest,
            extensions::get_extension_compatibility,
            extensions::get_all_extensions_compatibility,
            extensions::uninstall_extension,
            settings::get_app_settings,
            settings::save_app_settings,
            settings::reset_app_settings,
            auto_start::set_auto_start_enabled,
            auto_start::get_auto_start_enabled
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(browser_extension::run_server(app_handle));

            clipboard_history::init(app.handle().clone());
            file_search::init(app.handle().clone());
            downloads::init(app.handle().clone());

            app.manage(QuicklinkManager::new(app.handle())?);
            app.manage(FrecencyManager::new(app.handle())?);
            app.manage(SnippetManager::new(app.handle())?);
            app.manage(AiUsageManager::new(app.handle())?);
            app.manage(AiPresetManager::new(app.handle())?);
            app.manage(AliasManager::new(app.handle())?);
            app.manage(ScriptCommandManager::new(app.handle()));
            app.manage(FloatingNotesManager::new(app.handle())?);
            app.manage(settings::SettingsManager::new(app.handle())?);

            // Initialize hotkey manager
            let hotkey_manager = hotkey_manager::HotkeyManager::new(app.handle())?;

            // Load and register saved hotkeys
            if let Ok(saved_hotkeys) = hotkey_manager.get_all_hotkeys() {
                tracing::info!("Loading {} saved hotkeys", saved_hotkeys.len());

                for config in saved_hotkeys {
                    if let Some(mods) = hotkey_manager::modifiers_from_bits(config.modifiers) {
                        if let Some(code) = hotkey_manager::string_to_code(&config.key) {
                            let shortcut =
                                tauri_plugin_global_shortcut::Shortcut::new(Some(mods), code);
                            if let Err(e) = hotkey_manager.register_shortcut(
                                app.handle(),
                                config.command_id.clone(),
                                shortcut,
                            ) {
                                tracing::error!(
                                    "Failed to register hotkey for {}: {}",
                                    config.command_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }

            app.manage(hotkey_manager);

            setup_background_refresh(app.handle().clone());
            if let Err(e) = setup_global_shortcut(app) {
                tracing::error!(error = %e, "Failed to set up global shortcut");
            }
            setup_input_listener(app.handle());

            let soulver_core_path = app
                .path()
                .resource_dir()
                .unwrap()
                .join("SoulverWrapper/Vendor/SoulverCore-linux");

            soulver::initialize(soulver_core_path.to_str().unwrap());

            let quit_i = MenuItem::with_id(app.handle(), "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app.handle(), "show", "Show Flare", true, None::<&str>)?;
            let menu = Menu::with_items(app.handle(), &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app.handle())?;

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
                        tracing::info!("Window lost focus");
                        if let Some(window) = app.get_webview_window("main") {
                            // Check if close on blur is enabled in settings
                            if let Some(settings_manager) =
                                app.try_state::<settings::SettingsManager>()
                            {
                                match settings_manager.get_settings() {
                                    Ok(settings) => {
                                        tracing::info!(
                                            "Close on blur setting: {}",
                                            settings.close_on_blur
                                        );
                                        if settings.close_on_blur {
                                            tracing::info!("Hiding window due to close on blur");
                                            let _ = window.hide();
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to get settings: {}", e);
                                    }
                                }
                            } else {
                                tracing::warn!("Settings manager not available");
                                if !cfg!(debug_assertions) {
                                    // Fallback: hide in release mode if settings not available
                                    let _ = window.hide();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

// ============================================================================
// dmenu Mode
// ============================================================================

#[tauri::command]
fn dmenu_get_items() -> Vec<String> {
    DMENU_SESSION
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.items.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn dmenu_get_prompt() -> String {
    DMENU_SESSION
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.prompt.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn dmenu_get_case_insensitive() -> bool {
    DMENU_SESSION
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.case_insensitive)
        .unwrap_or(false)
}

#[tauri::command]
fn dmenu_select_item(item: String) {
    if let Some(session) = DMENU_SESSION
        .lock()
        .expect("dmenu session mutex poisoned")
        .as_ref()
    {
        session.output_selection(&item);
    }
    std::process::exit(0);
}

#[tauri::command]
fn dmenu_cancel() {
    std::process::exit(1);
}

/// Entry point for dmenu mode - runs a minimal Tauri app for menu selection
pub fn run_dmenu(session: DmenuSession) {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Store the session in global state
    *DMENU_SESSION.lock().expect("dmenu session mutex poisoned") = Some(session);

    tracing::info!("Starting Flare in dmenu mode");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            dmenu_get_items,
            dmenu_get_prompt,
            dmenu_get_case_insensitive,
            dmenu_select_item,
            dmenu_cancel
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();

                // Delay the event emission to ensure WebView is ready
                let window_clone = window.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    let _ = window_clone.emit("dmenu-mode", ());
                });
            } else {
                tracing::error!("dmenu: main window not found");
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building dmenu app");

    app.run(|_app, event| {
        if let tauri::RunEvent::WindowEvent { label, event, .. } = event {
            if label == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        // Cancel on window close
                        std::process::exit(1);
                    }
                    // Don't exit on focus loss - let the user press Escape to cancel
                    // This was causing immediate exit on window show
                    _ => {}
                }
            }
        }
    });
}
