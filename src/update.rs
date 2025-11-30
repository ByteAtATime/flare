use iced::widget::operation;
use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};
#[cfg(target_os = "linux")]
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::apps;
use crate::components::action_panel;
use crate::components::actions::ActionPanelItem;
use crate::extensions;
use crate::image_cache;
use crate::message::Message;
use crate::runtime;
use crate::screens::{Screen, Shell};
use crate::state::State;
use crate::types::{RustResponse, SidecarRequest, SidecarResponse};

use crate::clipboard;
use crate::oauth;
use crate::storage;

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::EscapePressed => {
            return handle_escape(state);
        }
        Message::WindowOpened(id) => {
            state.window_id = Some(id);
            if state.screen.can_search() {
                if state.screen.can_search() {
                    return operation::focus(state.search_input_id.clone());
                }
            }
        }
        Message::WindowClosed(id) => {
            if state.window_id == Some(id) {
                state.window_id = None;
            }
            if state.settings_window_id == Some(id) {
                state.settings_window_id = None;
            }
        }
        Message::ToggleWindow => {
            if let Some(id) = state.window_id {
                return window::close(id);
            } else {
                #[cfg(target_os = "linux")]
                {
                    if state.flare_settings.use_layer_shell {
                        let id = window::Id::unique();
                        state.window_id = Some(id);
                        return Task::done(Message::NewLayerShell {
                            settings: NewLayerShellSettings {
                                size: Some((774, 474)),
                                anchor: Anchor::empty(),
                                layer: Layer::Overlay,
                                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                                ..Default::default()
                            },
                            id,
                        });
                    } else {
                        let (id, open) = window::open(window::Settings {
                            decorations: false,
                            level: window::Level::AlwaysOnTop,
                            resizable: false,
                            size: iced::Size::new(774.0, 474.0),
                            ..Default::default()
                        });
                        return open.map(move |_| Message::WindowOpened(id));
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let (id, open) = window::open(window::Settings {
                        decorations: false,
                        level: window::Level::AlwaysOnTop,
                        resizable: false,
                        size: iced::Size::new(774.0, 474.0),
                        ..Default::default()
                    });
                    return open.map(move |_| Message::WindowOpened(id));
                }
            }
        }
        Message::OpenSettings => {
            if state.settings_window_id.is_some() {
                return Task::none();
            }
            let (id, open) = window::open(window::Settings {
                size: iced::Size::new(600.0, 400.0),
                resizable: true,
                ..Default::default()
            });
            return open.map(move |_| Message::SettingsWindowOpened(id));
        }
        Message::SettingsWindowOpened(id) => {
            state.settings_window_id = Some(id);
        }
        Message::UpdateTree(tree) => {
            if let Some(component) = tree.children.into_iter().next() {
                if let Some(mut new_screen) = Screen::new(component, Some(&state.screen)) {
                    if !state.search_text.is_empty() {
                        new_screen.on_search(&state.search_text);
                    }
                    state.screen = new_screen;
                    if state.screen.can_search() {
                        return operation::focus(state.search_input_id.clone());
                    }
                    state.update_selected_actions();
                    return state.screen.load_images();
                }
            }
        }
        Message::SearchTextChanged(text) => {
            state.search_text = text.clone();
            state.screen.on_search(&text);
            state.update_selected_actions();

            let mut tasks = vec![state.screen.load_images()];

            if let Some(callback) = state.screen.on_search_text_change() {
                tasks.push(send_callback(
                    state,
                    callback.id.clone(),
                    Value::String(text),
                ));
            }
            return Task::batch(tasks);
        }
        Message::DropdownChanged(value) => {
            state.screen.set_dropdown_value(&value);
            if let Some(dropdown) = state.screen.get_search_bar_accessory() {
                if let Some(callback) = &dropdown.props.on_change {
                    return send_callback(state, callback.id.clone(), Value::String(value));
                }
            }
        }
        Message::KeyPressed(key, modifiers) => {
            return handle_key_press(state, key, modifiers);
        }
        Message::ImageLoaded(url, handle) => {
            image_cache::set(url, handle);
        }
        Message::ImageLoadFailed(url) => {
            image_cache::clear_pending(&url);
        }
        Message::InvokeAction(handler) => {
            return handler.call();
        }
        Message::ActionPanel(msg) => {
            let actions = state.selected_actions.clone();
            let (task, command) = state.action_panel.update(msg, &actions);

            if !state.action_panel.visible && state.screen.can_search() {
                if command.is_none() {
                    return Task::batch(vec![
                        task,
                        operation::focus(state.search_input_id.clone()),
                    ]);
                }
            }

            if let Some(handler) = command {
                return Task::batch(vec![task, handler.call()]);
            }

            return task;
        }
        Message::Tick(now) => {
            if state.action_panel.animation.start_time.is_some() {
                return update(
                    state,
                    Message::ActionPanel(action_panel::Message::Tick(now)),
                );
            }
        }
        Message::ShowToast(message) => {
            state.toast_message = message;
        }
        Message::LaunchCommand(command) => {
            let entry = command
                .extension_path
                .join(format!("{}.js", command.command_name));
            let assets_path = command.extension_path.join("assets");

            let id = format!("ext:{}:{}", command.extension_name, command.command_name);
            state.frecency.visit(id);

            state.search_text.clear();
            let preferences = state
                .preferences
                .get_extension_preferences(&command.extension_name, &state.extensions);

            let entry_str = entry.to_string_lossy().to_string();
            let assets_str = assets_path.to_string_lossy().to_string();

            return Task::perform(
                async move {
                    runtime::launch_extension(&entry_str, &assets_str, preferences)
                        .await
                        .map(|(w, r)| (w, Arc::new(Mutex::new(r))))
                        .map_err(|e| e.to_string())
                },
                Message::ExtensionLaunched,
            );
        }
        Message::ExtensionLaunched(result) => match result {
            Ok((writer, reader)) => {
                state.writer = Some(writer);
                state.reader = Some(reader);
            }
            Err(e) => {
                eprintln!("Failed to launch extension: {:?}", e);
            }
        },
        Message::LaunchApp(app) => {
            let id = format!("app:{}", app.id);
            state.frecency.visit(id);

            apps::launch_application(&app);
            if let Some(id) = state.window_id {
                return window::close(id);
            }
        }
        Message::ResetFrecency(id) => {
            state.frecency.reset(&id);
            if let crate::screens::Screen::Root(root_screen) = &mut state.screen {
                root_screen.sort_items(&state.frecency);
            }
            state.update_selected_actions();
        }
        Message::PopToRoot => {
            state.writer = None;
            state.reader = None;

            state.search_text.clear();
            let commands = extensions::get_launchable_commands(&state.extensions);

            let mut root_screen =
                crate::screens::root::RootScreen::new(commands, state.apps.clone());
            root_screen.sort_items(&state.frecency);

            state.screen = Screen::Root(root_screen);
            state.update_selected_actions();
            if state.screen.can_search() {
                return operation::focus(state.search_input_id.clone());
            }
        }
        Message::Settings(settings_msg) => {
            use crate::screens::settings::SettingsMessage;
            match settings_msg {
                SettingsMessage::PreferenceChanged {
                    extension_id,
                    key,
                    value,
                } => {
                    state.preferences.set_value(&extension_id, &key, value);
                    if let Err(e) = state.preferences.save() {
                        eprintln!("Failed to save preferences: {}", e);
                    }
                }
                SettingsMessage::FlareSettingChanged { use_layer_shell } => {
                    state.flare_settings.use_layer_shell = use_layer_shell;
                    if let Err(e) = state.flare_settings.save() {
                        eprintln!("Failed to save Flare settings: {}", e);
                    }
                }
            }
        }
        Message::OpenUrl(url) => {
            let _ = crate::utils::open_url(&url);
        }
        Message::HandleOAuthRedirect(url) => {
            crate::deep_link::handle_oauth_redirect(&url);
        }

        Message::SidecarMessage(response) => {
            return handle_sidecar_response(state, response);
        }

        Message::SidecarOperationFinished(id, result) => {
            if let Some(writer) = &state.writer {
                let writer = writer.clone();
                return Task::perform(
                    async move {
                        let response = match result {
                            Ok(val) => RustResponse::Success { id, result: val },
                            Err(e) => RustResponse::Error { id, error: e },
                        };
                        let _ = writer.send(&response).await;
                    },
                    |_| Message::Tick(Instant::now()),
                );
            }
        }

        Message::RunCallback(id, args) => {
            return send_callback(state, id, args);
        }

        msg => return dispatch_screen_message(state, msg),
    }
    Task::none()
}

fn handle_sidecar_response(state: &mut State, response: SidecarResponse) -> Task<Message> {
    match response {
        SidecarResponse::Initialized { success, error } => {
            if !success {
                eprintln!("Plugin initialization failed: {:?}", error);
            }
            Task::none()
        }
        SidecarResponse::CallbackResult { success, error } => {
            if !success {
                eprintln!("Callback failed: {:?}", error);
            }
            Task::none()
        }
        SidecarResponse::ShowToast {
            id,
            title,
            message: _,
            style: _,
        } => {
            state.toast_message = title;
            Task::done(Message::SidecarOperationFinished(id, Ok(None)))
        }
        SidecarResponse::UpdateTree { id, tree } => {
            if let Some(component) = tree.children.into_iter().next() {
                if let Some(mut new_screen) = Screen::new(component, Some(&state.screen)) {
                    if !state.search_text.is_empty() {
                        new_screen.on_search(&state.search_text);
                    }
                    state.screen = new_screen;
                    state.update_selected_actions();
                    return Task::batch(vec![
                        state.screen.load_images(),
                        Task::done(Message::SidecarOperationFinished(id, Ok(None))),
                    ]);
                }
            }

            let focus_task = if state.screen.can_search() {
                operation::focus(state.search_input_id.clone())
            } else {
                Task::none()
            };

            Task::batch(vec![
                focus_task,
                Task::done(Message::SidecarOperationFinished(id, Ok(None))),
            ])
        }
        SidecarResponse::Pop { id } => Task::batch(vec![
            Task::done(Message::PopToRoot),
            Task::done(Message::SidecarOperationFinished(id, Ok(None))),
        ]),
        SidecarResponse::OpenExtensionPreferences { id }
        | SidecarResponse::OpenCommandPreferences { id } => Task::batch(vec![
            Task::done(Message::OpenSettings),
            Task::done(Message::SidecarOperationFinished(id, Ok(None))),
        ]),
        SidecarResponse::OpenUrl { id, url } => {
            let _ = crate::utils::open_url(&url);
            Task::done(Message::SidecarOperationFinished(id, Ok(None)))
        }

        SidecarResponse::LocalStorageSet {
            id,
            namespace,
            key,
            data,
        } => Task::perform(
            async move { storage::set(&namespace, &key, &data).map(|_| None::<serde_json::Value>) },
            move |res| Message::SidecarOperationFinished(id, res.map(|_| None)),
        ),
        SidecarResponse::LocalStorageGet { id, namespace, key } => {
            Task::perform(async move { storage::get(&namespace, &key) }, move |res| {
                Message::SidecarOperationFinished(id, Ok(res.map(Value::String)))
            })
        }
        SidecarResponse::LocalStorageRemove { id, namespace, key } => Task::perform(
            async move { storage::remove(&namespace, &key) },
            move |res| Message::SidecarOperationFinished(id, Ok(Some(Value::Bool(res)))),
        ),
        SidecarResponse::LocalStorageClear { id, namespace } => Task::perform(
            async move { storage::clear(&namespace).map(|_| None::<serde_json::Value>) },
            move |res| Message::SidecarOperationFinished(id, res.map(|_| None)),
        ),
        SidecarResponse::LocalStorageAll { id, namespace } => {
            Task::perform(async move { storage::get_all(&namespace) }, move |res| {
                Message::SidecarOperationFinished(id, Ok(Some(serde_json::to_value(res).unwrap())))
            })
        }

        SidecarResponse::ClipboardCopy {
            id,
            content,
            concealed,
        } => Task::perform(
            async move { clipboard::copy(content, concealed) },
            move |res| Message::SidecarOperationFinished(id, res),
        ),
        SidecarResponse::ClipboardClear { id } => {
            Task::perform(async move { clipboard::clear() }, move |res| {
                Message::SidecarOperationFinished(id, res)
            })
        }
        SidecarResponse::ClipboardRead { id, .. } => {
            Task::perform(async move { clipboard::read() }, move |res| {
                Message::SidecarOperationFinished(id, res)
            })
        }

        SidecarResponse::OAuthAuthorize {
            id,
            url,
            state: oauth_state,
        } => {
            if let Some(writer) = &state.writer {
                oauth::authorize(id, url, oauth_state, writer.clone());
            }
            Task::none()
        }
        SidecarResponse::OAuthSetTokens {
            id,
            provider_id,
            tokens,
        } => Task::perform(
            async move { oauth::set_tokens(provider_id, tokens) },
            move |res| Message::SidecarOperationFinished(id, res),
        ),
        SidecarResponse::OAuthGetTokens { id, provider_id } => {
            Task::perform(async move { oauth::get_tokens(provider_id) }, move |res| {
                Message::SidecarOperationFinished(id, res)
            })
        }
        SidecarResponse::OAuthRemoveTokens { id, provider_id } => Task::perform(
            async move { oauth::remove_tokens(provider_id) },
            move |res| Message::SidecarOperationFinished(id, res),
        ),
    }
}

fn send_callback(state: &mut State, callback_id: String, args: Value) -> Task<Message> {
    if let Some(writer) = &state.writer {
        let writer = writer.clone();
        let request = SidecarRequest::InvokeCallback { callback_id, args };

        return Task::perform(
            async move {
                let _ = writer.send_request(&request).await;
            },
            |_| Message::Tick(Instant::now()),
        );
    }
    Task::none()
}

fn dispatch_screen_message(state: &mut State, message: Message) -> Task<Message> {
    match (&mut state.screen, message) {
        (Screen::Root(_), Message::Root(crate::screens::root::RootMessage::RunAction(handler))) => {
            handler.call()
        }
        (Screen::Root(s), Message::Root(m)) => s.update(m).map(Message::Root),
        (Screen::Grid(s), Message::Grid(crate::screens::grid::GridMessage::Scrolled(vp))) => {
            let task = s
                .update(crate::screens::grid::GridMessage::Scrolled(vp))
                .map(Message::Grid);
            Task::batch(vec![task, s.load_images()])
        }
        (Screen::Grid(s), Message::Grid(m)) => s.update(m).map(Message::Grid),
        (Screen::List(s), Message::List(m)) => s.update(m).map(Message::List),
        (Screen::Detail(s), Message::Detail(m)) => s.update(m).map(Message::Detail),
        _ => Task::none(),
    }
}

fn handle_escape(state: &mut State) -> Task<Message> {
    if state.action_panel.visible {
        return update(state, Message::ActionPanel(action_panel::Message::Close));
    }

    if !state.search_text.is_empty() {
        state.search_text.clear();
        state.screen.on_search("");
        state.update_selected_actions();
        return operation::focus(state.search_input_id.clone());
    }

    if let Screen::Root(_) = &state.screen {
        if let Some(id) = state.window_id {
            return window::close(id);
        }
        return Task::none();
    }

    if let Some(writer) = &state.writer {
        let writer = writer.clone();
        return Task::perform(
            async move {
                let _ = writer.send_request(&SidecarRequest::Pop).await;
            },
            |_| Message::Tick(Instant::now()),
        );
    }

    Task::done(Message::PopToRoot)
}

fn handle_key_press(state: &mut State, key: Key, modifiers: Modifiers) -> Task<Message> {
    if let Key::Named(named_key) = key {
        if modifiers.is_empty() && named_key == Named::Escape {
            return handle_escape(state);
        }

        if state.action_panel.visible {
            if modifiers.is_empty() {
                match named_key {
                    Named::ArrowUp => {
                        return update(state, Message::ActionPanel(action_panel::Message::MoveUp));
                    }
                    Named::ArrowDown => {
                        return update(
                            state,
                            Message::ActionPanel(action_panel::Message::MoveDown),
                        );
                    }
                    Named::Enter => {
                        return update(
                            state,
                            Message::ActionPanel(action_panel::Message::InvokeSelected),
                        );
                    }
                    _ => {}
                }
            }
        }

        if named_key == Named::Enter {
            let index = match modifiers {
                m if m.is_empty() => Some(0),
                m if m == Modifiers::COMMAND => Some(1),
                _ => None,
            };

            if let Some(i) = index {
                let action = state
                    .selected_actions
                    .iter()
                    .flat_map(|item| match item {
                        ActionPanelItem::Action(a) => std::slice::from_ref(a).iter(),
                        ActionPanelItem::Section(s) => s.children.iter(),
                    })
                    .nth(i);

                if let Some(action) = action {
                    if let Some(handler) = &action.handler {
                        return handler.call();
                    }
                }
            }
            return Task::none();
        }
    }

    if modifiers == Modifiers::CTRL {
        if let Key::Character(c) = &key {
            if c == "," {
                return Task::done(Message::OpenSettings);
            }
        }
    }

    if modifiers == Modifiers::COMMAND {
        if let Key::Character(c) = &key {
            if c == "k" {
                if state.action_panel.visible {
                    return update(state, Message::ActionPanel(action_panel::Message::Close));
                } else {
                    return update(state, Message::ActionPanel(action_panel::Message::Open));
                }
            }
        }
    }

    let task = match &mut state.screen {
        Screen::Root(s) => s
            .update(crate::screens::root::RootMessage::KeyPressed(
                key, modifiers,
            ))
            .map(Message::Root),
        Screen::Grid(s) => s
            .update(crate::screens::grid::GridMessage::KeyPressed(
                key, modifiers,
            ))
            .map(Message::Grid),
        Screen::List(s) => s
            .update(crate::screens::list::ListMessage::KeyPressed(
                key, modifiers,
            ))
            .map(Message::List),
        Screen::Detail(s) => s
            .update(crate::screens::detail::DetailMessage::KeyPressed(
                key, modifiers,
            ))
            .map(Message::Detail),
    };

    state.update_selected_actions();
    task
}
