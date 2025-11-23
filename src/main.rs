mod cache;
mod components;
mod globals;
mod icons;
mod image_cache;
mod message;
mod position;
mod runtime;
mod screens;
mod types;

use globals::SCROLLABLE;
use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, StreamExt};
use iced::widget::{column, container, scrollable, stack, text_input};
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
    selected_actions: Vec<components::types::ActionPanelItem>,
    toast_message: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            screen: Screen::Grid(screens::grid::GridScreen::new(
                components::types::GridProps {
                    sections: vec![],
                    columns: None,
                    on_search_text_change: None,
                },
                None,
            )),
            search_text: String::new(),
            action_panel_visible: false,
            selected_actions: Vec::new(),
            toast_message: String::new(),
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let search_bar = if state.screen.can_search() {
        Some(
            container(
                text_input("Search...", &state.search_text)
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
                    }),
            )
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
    };

    let mut base_col = column![];
    if let Some(sb) = search_bar {
        base_col = base_col.push(sb);
    }

    base_col = base_col
        .push(
            scrollable(content)
                .height(Length::Fill)
                .id(SCROLLABLE.clone())
                .on_scroll(Message::Scrolled),
        )
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
            match first {
                Some(components::types::Component::Grid(grid)) => {
                    let vp = match &state.screen {
                        Screen::Grid(gs) => gs.get_viewport(),
                        _ => None,
                    };
                    state.screen = Screen::Grid(screens::grid::GridScreen::new(grid.clone(), vp));
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
        Message::KeyPressed(key, modifiers) => {
            use iced::keyboard::{Key, Modifiers, key::Named};
            if let Key::Named(named_key) = key {
                if modifiers.is_empty() && named_key == Named::Escape {
                    if state.action_panel_visible {
                        state.action_panel_visible = false;
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
                                components::types::ActionPanelItem::Action(action) => {
                                    std::slice::from_ref(action).iter()
                                }
                                components::types::ActionPanelItem::Section(section) => {
                                    section.children.iter()
                                }
                            })
                            .nth(i)
                        {
                            if let Some(callback) = &action.on_action {
                                if let Some(mut sender) =
                                    globals::RUNTIME_SENDER.lock().unwrap().clone()
                                {
                                    let callback_id = callback.id.clone();
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
        Message::ToggleActionPanel(visibility) => {
            state.action_panel_visible = visibility;
        }
        Message::Scrolled(viewport) => {
            if let Screen::Grid(grid_screen) = &mut state.screen {
                let _ = grid_screen.update(screens::grid::GridMessage::Scrolled(viewport));
            }
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

            // apparently this prevents too many threads, i have no idea how it works it's above my paygrade
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
