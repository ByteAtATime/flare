use iced::widget::operation;
use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};
use serde_json::Value;
use std::time::Instant;

use crate::apps;
use crate::components::action_panel;
use crate::components::actions::ActionPanelItem;
use crate::extensions;
use crate::globals;
use crate::image_cache;
use crate::message::Message;
use crate::runtime;
use crate::screens::{Screen, Shell};
use crate::screens::{detail, grid, list, root};
use crate::state::State;
use crate::types::SidecarRequest;

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::EscapePressed => {
            return handle_escape(state);
        }
        Message::WindowOpened(id) => {
            state.window_id = Some(id);
            if state.screen.can_search() {
                return operation::focus(state.search_input_id.clone());
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
                let (id, open) = window::open(window::Settings {
                    decorations: false,
                    level: window::Level::AlwaysOnTop,
                    resizable: false,
                    // TODO: these sizes are copied from raycast
                    size: iced::Size::new(774.0, 474.0),
                    ..Default::default()
                });
                return open.map(move |_| Message::WindowOpened(id));
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
                }
            }
        }
        Message::SearchTextChanged(text) => {
            state.search_text = text.clone();
            state.screen.on_search(&text);
            state.update_selected_actions();
            if let Some(callback) = state.screen.on_search_text_change() {
                globals::send_callback(callback.id.clone(), Value::String(text));
            }
        }
        Message::DropdownChanged(value) => {
            state.screen.set_dropdown_value(&value);
            if let Some(dropdown) = state.screen.get_search_bar_accessory() {
                if let Some(callback) = &dropdown.props.on_change {
                    globals::send_callback(callback.id.clone(), Value::String(value));
                }
            }
        }
        Message::KeyPressed(key, modifiers) => {
            return handle_key_press(state, key, modifiers);
        }
        Message::ImageLoaded(url, handle) => {
            image_cache::set(url, handle);
        }
        Message::InvokeAction(handler) => {
            return handler.call();
        }
        Message::ToggleActionPanel(visibility) => {
            state.action_panel_visible = visibility;
            if visibility {
                state.action_panel_selected = 0;
                state.action_panel_start_time = Some(Instant::now());
                state.action_panel_opacity = action_panel::animation::OPACITY_START;
                state.action_panel_scale = action_panel::animation::SCALE_START;
                return operation::focus_next();
            } else {
                state.action_panel_start_time = None;
                state.action_panel_search.clear();
                state.action_panel_selected = 0;
                if state.screen.can_search() {
                    return operation::focus(state.search_input_id.clone());
                }
            }
        }
        Message::Tick(now) => {
            if let Some(start) = state.action_panel_start_time {
                let elapsed = now.duration_since(start).as_millis() as f32;
                let duration = action_panel::animation::DURATION_MS as f32;
                let t = (elapsed / duration).clamp(0.0, 1.0);

                let ease = 1.0 - (1.0 - t).powi(2);

                state.action_panel_opacity = action_panel::animation::OPACITY_START
                    + (1.0 - action_panel::animation::OPACITY_START) * ease;
                state.action_panel_scale = action_panel::animation::SCALE_START
                    + (1.0 - action_panel::animation::SCALE_START) * ease;

                if t >= 1.0 {
                    state.action_panel_start_time = None;
                    state.action_panel_opacity = 1.0;
                    state.action_panel_scale = 1.0;
                }
            }
        }
        Message::ActionPanelSearchChanged(text) => {
            state.action_panel_search = text;
            state.action_panel_selected = 0;
        }
        Message::ActionPanelSelect(index) => {
            state.action_panel_selected = index;
        }
        Message::ActionPanelMoveUp => {
            if state.action_panel_selected > 0 {
                state.action_panel_selected -= 1;
            }
        }
        Message::ActionPanelMoveDown => {
            let count = crate::components::action_panel::count_actions(
                &state.selected_actions,
                &state.action_panel_search,
            );
            if state.action_panel_selected < count.saturating_sub(1) {
                state.action_panel_selected += 1;
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
            if let Err(e) = runtime::launch_extension(
                &entry.to_string_lossy(),
                &assets_path.to_string_lossy(),
                preferences,
            ) {
                eprintln!("Failed to launch extension: {:?}", e);
            }
        }
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
            if let Screen::Root(root_screen) = &mut state.screen {
                root_screen.sort_items(&state.frecency);
            }
            state.update_selected_actions();
        }
        Message::PopToRoot => {
            runtime::stop_runtime();
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
            }
        }
        Message::OpenUrl(url) => {
            let _ = crate::utils::open_url(&url);
        }
        msg => return dispatch_screen_message(state, msg),
    }
    Task::none()
}

fn dispatch_screen_message(state: &mut State, message: Message) -> Task<Message> {
    match (&mut state.screen, message) {
        (Screen::Root(_), Message::Root(root::RootMessage::RunAction(handler))) => handler.call(),
        (Screen::Root(s), Message::Root(m)) => s.update(m).map(Message::Root),
        (Screen::Grid(s), Message::Grid(m)) => s.update(m).map(Message::Grid),
        (Screen::List(s), Message::List(m)) => s.update(m).map(Message::List),
        (Screen::Detail(s), Message::Detail(m)) => s.update(m).map(Message::Detail),
        _ => Task::none(),
    }
}

fn handle_escape(state: &mut State) -> Task<Message> {
    if state.action_panel_visible {
        state.action_panel_visible = false;
        state.action_panel_search.clear();
        state.action_panel_selected = 0;
        state.action_panel_start_time = None;
        if state.screen.can_search() {
            return operation::focus(state.search_input_id.clone());
        }
        return Task::none();
    }

    if !state.search_text.is_empty() {
        state.search_text.clear();
        state.screen.on_search("");
        state.update_selected_actions();
        return operation::focus(state.search_input_id.clone());
    }

    if let Screen::Root(_) = &state.screen {
        return Task::none();
    }

    if let Some(runtime) = globals::RUNTIME.lock().unwrap().as_mut() {
        let _ = runtime.send_request(&SidecarRequest::Pop);
        return Task::none();
    }

    Task::done(Message::PopToRoot)
}

fn handle_key_press(state: &mut State, key: Key, modifiers: Modifiers) -> Task<Message> {
    if let Key::Named(named_key) = key {
        if modifiers.is_empty() && named_key == Named::Escape {
            return handle_escape(state);
        }

        if state.action_panel_visible {
            if modifiers.is_empty() {
                match named_key {
                    Named::ArrowUp => {
                        return Task::done(Message::ActionPanelMoveUp);
                    }
                    Named::ArrowDown => {
                        return Task::done(Message::ActionPanelMoveDown);
                    }
                    Named::Enter => {
                        let filtered = crate::components::action_panel::filter_actions(
                            &state.selected_actions,
                            &state.action_panel_search,
                        );
                        let action = filtered
                            .iter()
                            .flat_map(|item| match item {
                                ActionPanelItem::Action(a) => std::slice::from_ref(a).iter(),
                                ActionPanelItem::Section(s) => s.children.iter(),
                            })
                            .nth(state.action_panel_selected);

                        if let Some(action) = action {
                            if let Some(handler) = &action.handler {
                                state.action_panel_visible = false;
                                state.action_panel_search.clear();
                                state.action_panel_selected = 0;
                                state.action_panel_start_time = None;
                                return handler.call();
                            }
                        }
                        return Task::none();
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
                let new_vis = !state.action_panel_visible;
                state.action_panel_visible = new_vis;
                if new_vis {
                    state.action_panel_selected = 0;
                    state.action_panel_start_time = Some(Instant::now());
                    state.action_panel_opacity = 0.9;
                    state.action_panel_scale = action_panel::animation::SCALE_START;
                    return operation::focus_next();
                } else {
                    state.action_panel_start_time = None;
                    if state.screen.can_search() {
                        return operation::focus(state.search_input_id.clone());
                    } else {
                        return Task::none();
                    }
                }
            }
        }
    }

    let task = match &mut state.screen {
        Screen::Root(s) => s
            .update(root::RootMessage::KeyPressed(key, modifiers))
            .map(Message::Root),
        Screen::Grid(s) => s
            .update(grid::GridMessage::KeyPressed(key, modifiers))
            .map(Message::Grid),
        Screen::List(s) => s
            .update(list::ListMessage::KeyPressed(key, modifiers))
            .map(Message::List),
        Screen::Detail(s) => s
            .update(detail::DetailMessage::KeyPressed(key, modifiers))
            .map(Message::Detail),
    };

    state.update_selected_actions();
    task
}
