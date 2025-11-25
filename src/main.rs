mod cache;
mod components;
mod extensions;
mod globals;
mod icons;
mod image_cache;
mod message;
mod position;
mod runtime;
mod screens;
mod types;
mod utils;

use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, StreamExt};
use iced::widget::{column, container, pick_list, row, stack, text_input};
use iced::{Element, Length, Subscription, Task, Theme};
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::components::actions::render_action_panel;
use crate::message::Message;
use crate::screens::detail::DetailMessage;
use crate::screens::{Screen, Shell};

struct State {
    screen: Screen,
    search_text: String,
    action_panel_visible: bool,
    selected_actions: Vec<components::actions::ActionPanelItem>,
    toast_message: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            screen: Screen::Grid(screens::grid::GridScreen::new(
                components::types::GridProps {
                    sections: vec![],
                    props: components::grid::GridProperties {
                        columns: None,
                        on_search_text_change: None,
                        search_bar_accessory: None,
                    },
                },
                None,
                None,
            )),
            search_text: String::new(),
            action_panel_visible: false,
            selected_actions: Vec::new(),
            toast_message: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DropdownOption {
    title: String,
    value: String,
}

impl std::fmt::Display for DropdownOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let search_bar = if state.screen.can_search() {
        let text_input = text_input("Search...", &state.search_text)
            .on_input(Message::SearchTextChanged)
            .size(20)
            .padding(12)
            .style(|_theme: &Theme, status| {
                let base = text_input::default(_theme, status);
                text_input::Style {
                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                    border: iced::Border::default(),
                    ..base
                }
            });

        let accessory = if let Some(dropdown) = state.screen.get_search_bar_accessory() {
            let options: Vec<DropdownOption> = dropdown
                .children
                .iter()
                .flat_map(|child| match child {
                    components::dropdown::DropdownChild::GridItem(item)
                    | components::dropdown::DropdownChild::ListItem(item) => {
                        vec![DropdownOption {
                            title: item.props.title.clone(),
                            value: item.props.value.clone(),
                        }]
                    }
                    components::dropdown::DropdownChild::GridSection(section)
                    | components::dropdown::DropdownChild::ListSection(section) => section
                        .children
                        .iter()
                        .map(|item| DropdownOption {
                            title: item.props.title.clone(),
                            value: item.props.value.clone(),
                        })
                        .collect(),
                })
                .collect();

            let selected_value = dropdown
                .props
                .value
                .as_ref()
                .or(dropdown.props.default_value.as_ref());

            let selected = selected_value
                .and_then(|val| options.iter().find(|opt| &opt.value == val).cloned())
                .or_else(|| options.first().cloned());

            Some(
                pick_list(options, selected, |opt| Message::DropdownChanged(opt.value)).padding(10),
            )
        } else {
            None
        };

        let mut row_content = row![text_input].align_y(iced::Alignment::Center);

        if let Some(acc) = accessory {
            row_content = row_content.push(acc);
        }

        Some(
            container(row_content)
                .padding(10)
                .style(|_theme: &Theme| container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb8(0x33, 0x33, 0x33),
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
    } else {
        None
    };

    let content = match &state.screen {
        Screen::Grid(grid_screen) => grid_screen.view().map(Message::Grid),
        Screen::Detail(detail_screen) => detail_screen.view().map(Message::Detail),
        Screen::List(list_screen) => list_screen.view().map(Message::List),
    };

    let mut base_col = column![];
    if let Some(sb) = search_bar {
        base_col = base_col.push(sb);
    }

    base_col = base_col
        .push(container(content).width(Length::Fill).height(Length::Fill))
        .push(components::footer::render_footer(state));

    container(if state.action_panel_visible {
        stack![base_col, render_action_panel(state)].into()
    } else {
        Element::from(base_col)
    })
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UpdateTree(tree) => {
            let first = tree.children.first();

            let current_dropdown_value = state
                .screen
                .get_search_bar_accessory()
                .and_then(|d| d.props.value.clone());

            match first {
                Some(components::types::Component::Grid(grid)) => {
                    let (vp, id) = match &state.screen {
                        Screen::Grid(gs) => (gs.get_viewport(), Some(gs.scrollable_id.clone())),
                        _ => (None, None),
                    };

                    let mut new_props = grid.clone();
                    if let Some(acc) = new_props.props.search_bar_accessory.as_mut() {
                        if acc.props.value.is_none() {
                            acc.props.value = current_dropdown_value;
                        }
                    }

                    let mut screen = screens::grid::GridScreen::new(new_props, vp, id);

                    if !state.search_text.is_empty() {
                        screen.on_search(&state.search_text);
                    }

                    state.screen = Screen::Grid(screen);
                    update_selected_actions(state);
                }
                Some(components::types::Component::List(list)) => {
                    let (vp, id) = match &state.screen {
                        Screen::List(ls) => (ls.get_viewport(), Some(ls.scrollable_id.clone())),
                        _ => (None, None),
                    };

                    let mut new_props = list.clone();
                    if let Some(acc) = new_props.props.search_bar_accessory.as_mut() {
                        if acc.props.value.is_none() {
                            acc.props.value = current_dropdown_value;
                        }
                    }

                    let mut screen = screens::list::ListScreen::new(new_props, vp, id);

                    if !state.search_text.is_empty() {
                        screen.on_search(&state.search_text);
                    }

                    state.screen = Screen::List(screen);
                    update_selected_actions(state);
                }
                Some(components::types::Component::Detail(detail)) => {
                    state.screen =
                        Screen::Detail(screens::detail::DetailScreen::new(detail.clone()));
                    update_selected_actions(state);
                }
                _ => {}
            }
        }
        Message::SearchTextChanged(text) => {
            state.search_text = text.clone();
            state.screen.on_search(&text);
            update_selected_actions(state);
        }
        Message::DropdownChanged(value) => {
            state.screen.set_dropdown_value(&value);

            if let Some(dropdown) = state.screen.get_search_bar_accessory() {
                if let Some(callback) = &dropdown.props.on_change {
                    if let Some(mut sender) = globals::RUNTIME_SENDER.lock().unwrap().clone() {
                        let callback_id = callback.id.clone();
                        std::thread::spawn(move || {
                            futures::executor::block_on(async move {
                                sender
                                    .send((callback_id, serde_json::Value::String(value)))
                                    .await
                                    .ok();
                            });
                        });
                    }
                }
            }
        }
        Message::KeyPressed(key, modifiers) => {
            use iced::keyboard::{Key, Modifiers, key::Named};
            if let Key::Named(named_key) = key {
                if modifiers.is_empty() && named_key == Named::Escape {
                    if state.action_panel_visible {
                        state.action_panel_visible = false;
                        return Task::none();
                    }

                    if let Some(runtime) = globals::RUNTIME.lock().unwrap().as_mut() {
                        let _ = runtime.send_request(&types::SidecarRequest::Pop);
                        return Task::none();
                    }
                }

                if named_key == Named::Enter {
                    let index = if modifiers.is_empty() {
                        Some(0)
                    } else if modifiers == iced::keyboard::Modifiers::COMMAND {
                        Some(1)
                    } else {
                        None
                    };

                    if let Some(i) = index {
                        if let Some(action) = state
                            .selected_actions
                            .iter()
                            .flat_map(|item| match item {
                                components::actions::ActionPanelItem::Action(action) => {
                                    std::slice::from_ref(action).iter()
                                }
                                components::actions::ActionPanelItem::Section(section) => {
                                    section.children.iter()
                                }
                            })
                            .nth(i)
                        {
                            if let Some(callback) = &action.props.on_action {
                                execute_callback(callback.id.clone());
                            }
                        }
                    }
                    return Task::none();
                }
            }

            if modifiers == Modifiers::COMMAND {
                if let Key::Character(c) = key.clone() {
                    if c == "k" {
                        state.action_panel_visible = !state.action_panel_visible;
                        return Task::none();
                    }
                }
            }

            match &mut state.screen {
                Screen::Grid(grid_screen) => {
                    let result = grid_screen
                        .update(screens::grid::GridMessage::KeyPressed(key, modifiers))
                        .map(Message::Grid);
                    update_selected_actions(state);
                    return result;
                }
                Screen::List(list_screen) => {
                    let result = list_screen
                        .update(screens::list::ListMessage::KeyPressed(key, modifiers))
                        .map(Message::List);
                    update_selected_actions(state);
                    return result;
                }
                Screen::Detail(detail_screen) => {
                    let result = detail_screen
                        .update(DetailMessage::KeyPressed(key, modifiers))
                        .map(Message::Detail);
                    update_selected_actions(state);
                    return result;
                }
            }
        }
        Message::ImageLoaded(url, handle) => {
            image_cache::set(url, handle);
        }
        Message::InvokeAction(callback_id) => {
            execute_callback(callback_id);
        }
        Message::ToggleActionPanel(visibility) => {
            state.action_panel_visible = visibility;
        }
        Message::ShowToast(message) => {
            state.toast_message = message;
        }

        Message::Grid(grid_message) => match &mut state.screen {
            Screen::Grid(grid_screen) => {
                let result = grid_screen.update(grid_message).map(Message::Grid);
                update_selected_actions(state);
                return result;
            }
            _ => {}
        },
        Message::List(list_message) => match &mut state.screen {
            Screen::List(list_screen) => {
                let result = list_screen.update(list_message).map(Message::List);
                update_selected_actions(state);
                return result;
            }
            _ => {}
        },
        Message::Detail(detail_message) => match &mut state.screen {
            Screen::Detail(detail_screen) => {
                let result = detail_screen.update(detail_message).map(Message::Detail);
                update_selected_actions(state);
                return result;
            }
            _ => {}
        },
    }
    Task::none()
}

fn execute_callback(callback_id: String) {
    if let Some(mut sender) = globals::RUNTIME_SENDER.lock().unwrap().clone() {
        std::thread::spawn(move || {
            futures::executor::block_on(async move {
                sender
                    .send((callback_id, serde_json::Value::Null))
                    .await
                    .ok();
            });
        });
    }
}

fn update_selected_actions(state: &mut State) {
    if let Some(action_panel) = state.screen.get_action_panel() {
        state.selected_actions = action_panel.children.clone();
    } else {
        state.selected_actions.clear();
    }
}

fn message_stream() -> impl futures::Stream<Item = Message> {
    let receiver = {
        let mut guard = globals::RECEIVER.lock().unwrap();
        guard.take()
    };

    futures::stream::unfold(receiver, |state| async move {
        if let Some(mut receiver) = state {
            if let Some(msg) = receiver.next().await {
                return Some((msg, Some(receiver)));
            }
        } else {
            futures::future::pending::<()>().await;
        }
        None
    })
}

fn subscription(_state: &State) -> Subscription<Message> {
    let keyboard_sub =
        iced::keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)));

    let message_sub = Subscription::run(message_stream);

    Subscription::batch(vec![message_sub, keyboard_sub])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    extensions::scan_extensions();

    let (sender, receiver) = mpsc::unbounded();
    *globals::SENDER.lock().unwrap() = Some(sender);
    *globals::RECEIVER.lock().unwrap() = Some(receiver);

    let (callback_sender, callback_receiver) = mpsc::unbounded::<(String, Value)>();
    *globals::RUNTIME_SENDER.lock().unwrap() = Some(callback_sender);

    let (image_sender, image_receiver) = std::sync::mpsc::channel::<String>();
    *globals::IMAGE_LOADER.lock().unwrap() = Some(image_sender);

    let shared_receiver = Arc::new(Mutex::new(image_receiver));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(20)
                .tcp_nodelay(true)
                .build()
                .unwrap();

            let semaphore = Arc::new(tokio::sync::Semaphore::new(10));

            loop {
                let url = {
                    let lock = shared_receiver.lock().unwrap();
                    match lock.recv() {
                        Ok(u) => u,
                        Err(_) => break,
                    }
                };

                let client = client.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _permit = permit;

                    match image_cache::fetch_and_cache(&client, url.clone()).await {
                        Ok(handle) => {
                            let sender = globals::SENDER.lock().unwrap().clone();
                            if let Some(mut s) = sender {
                                let _ = s.send(Message::ImageLoaded(url, handle)).await;
                            }
                        }
                        Err(_) => {
                            image_cache::clear_pending(&url);
                        }
                    }
                });
            }
        });
    });

    std::thread::spawn(move || {
        runtime::setup_and_run(callback_receiver);
    });

    iced::application(|| State::default(), update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .font(include_bytes!("./assets/icons.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}
