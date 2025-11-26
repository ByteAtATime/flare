use iced::{
    Task,
    keyboard::{Key, Modifiers, key::Named},
    window,
};
use serde_json::Value;

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
        Message::WindowOpened(id) => {
            state.window_id = Some(id);
        }
        Message::WindowClosed(id) => {
            if state.window_id == Some(id) {
                state.window_id = None;
            }
        }
        Message::ToggleWindow => {
            if let Some(id) = state.window_id {
                return window::close(id);
            } else {
                let (id, open) = window::open(window::Settings::default());
                return open.map(move |_| Message::WindowOpened(id));
            }
        }
        Message::UpdateTree(tree) => {
            if let Some(component) = tree.children.into_iter().next() {
                if let Some(mut new_screen) = Screen::new(component, Some(&state.screen)) {
                    if !state.search_text.is_empty() {
                        new_screen.on_search(&state.search_text);
                    }
                    state.screen = new_screen;
                    state.update_selected_actions();
                }
            }
        }
        Message::SearchTextChanged(text) => {
            state.search_text = text.clone();
            state.screen.on_search(&text);
            state.update_selected_actions();
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
        Message::InvokeAction(callback_id) => {
            globals::send_callback(callback_id, Value::Null);
        }
        Message::ToggleActionPanel(visibility) => {
            state.action_panel_visible = visibility;
        }
        Message::ShowToast(message) => {
            state.toast_message = message;
        }
        Message::LaunchCommand(command) => {
            let entry = command
                .extension_path
                .join(format!("{}.js", command.command_name));
            state.search_text.clear();
            if let Err(e) = runtime::launch_extension(&entry.to_string_lossy()) {
                eprintln!("Failed to launch extension: {:?}", e);
            }
        }
        Message::PopToRoot => {
            runtime::stop_runtime();
            state.search_text.clear();
            let commands = extensions::get_launchable_commands(&state.extensions);
            state.screen = Screen::Root(crate::screens::root::RootScreen::new(commands));
        }
        msg => return dispatch_screen_message(state, msg),
    }
    Task::none()
}

fn dispatch_screen_message(state: &mut State, message: Message) -> Task<Message> {
    match (&mut state.screen, message) {
        (Screen::Root(s), Message::Root(m)) => s.update(m).map(Message::Root),
        (Screen::Grid(s), Message::Grid(m)) => s.update(m).map(Message::Grid),
        (Screen::List(s), Message::List(m)) => s.update(m).map(Message::List),
        (Screen::Detail(s), Message::Detail(m)) => s.update(m).map(Message::Detail),
        _ => Task::none(),
    }
}

fn handle_key_press(state: &mut State, key: Key, modifiers: Modifiers) -> Task<Message> {
    if let Key::Named(named_key) = key {
        if modifiers.is_empty() && named_key == Named::Escape {
            if state.action_panel_visible {
                state.action_panel_visible = false;
                return Task::none();
            }

            if let Screen::Root(_) = &state.screen {
                return Task::none();
            }

            if let Some(runtime) = globals::RUNTIME.lock().unwrap().as_mut() {
                let _ = runtime.send_request(&SidecarRequest::Pop);
                return Task::none();
            }

            return Task::done(Message::PopToRoot);
        }

        if named_key == Named::Enter {
            if let Screen::Root(root) = &state.screen {
                if let Some(cmd) = root.get_selected_command() {
                    return Task::done(Message::LaunchCommand(cmd.clone()));
                }
                return Task::none();
            }

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
                    if let Some(cb) = &action.props.on_action {
                        globals::send_callback(cb.id.clone(), Value::Null);
                    }
                }
            }
            return Task::none();
        }
    }

    if modifiers == Modifiers::COMMAND {
        if let Key::Character(c) = &key {
            if c == "k" {
                state.action_panel_visible = !state.action_panel_visible;
                return Task::none();
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
