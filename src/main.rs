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

// use components::actions::render_action_panel;
// use components::footer::render_footer;
use globals::SCROLLABLE;
use iced::futures::channel::mpsc;
use iced::futures::{self, SinkExt, StreamExt};
use iced::widget::{column, container, scrollable, text_input};
use iced::{Element, Length, Subscription, Task, Theme};
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::message::Message;
use crate::screens::{Screen, Shell};

struct State {
    screen: Screen,
    search_text: String,
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
            )),
            search_text: String::new(),
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    // let content = state
    //     .filtered_tree
    //     .as_ref()
    //     .map(|tree| {
    //         tree.children
    //             .iter()
    //             .fold(column![].height(Length::Shrink), |col, child| {
    //                 col.push(components::render_component(
    //                     child,
    //                     state.selected_index,
    //                     POSITION_TRACKER.clone(),
    //                     state.viewport.as_ref(),
    //                 ))
    //             })
    //     })
    //     .unwrap_or_else(|| column![].height(Length::Shrink));

    let search_bar = container(
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
    });

    // let base = column![
    //     search_bar,
    //     scrollable(content)
    //         .height(Length::Fill)
    //         .id(SCROLLABLE.clone())
    //         .on_scroll(Message::Scrolled),
    //     render_footer(state)
    // ];

    // container(if state.action_panel_visible {
    //     stack![base, render_action_panel(state)].into()
    // } else {
    //     Element::from(base)
    // })
    // .into()

    let content = match &state.screen {
        Screen::Grid(grid_screen) => grid_screen.view().map(Message::Grid),
    };

    column![
        search_bar,
        scrollable(content)
            .height(Length::Fill)
            .id(SCROLLABLE.clone()),
    ]
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UpdateTree(tree) => {
            let first = tree.children.first();
            if let Some(components::types::Component::Grid(grid)) = first {
                state.screen = Screen::Grid(screens::grid::GridScreen::new(grid.clone()));
            }
        }
        Message::SearchTextChanged(text) => {
            state.search_text = text.clone();
            state.screen.on_search(&text);
        }
        Message::KeyPressed(key, modifiers) => match &mut state.screen {
            Screen::Grid(grid_screen) => {
                grid_screen.update(screens::grid::GridMessage::KeyPressed(key, modifiers))
            }
        },
        Message::ImageLoaded(url, handle) => {
            image_cache::set(url, handle);
        }
        Message::Grid(grid_message) => match &mut state.screen {
            Screen::Grid(grid_screen) => grid_screen.update(grid_message),
        },
        _ => {}
    }
    // match message {
    //     Message::UpdateToast(new_message) => {
    //         state.toast_message = new_message;
    //         Task::none()
    //     }
    //     Message::UpdateTree(tree) => {
    //         state.update_tree(tree);
    //         Task::none()
    //     }
    //     Message::SearchTextChanged(text) => {
    //         if let Some(callback_id) = state.update_search(text.clone()) {
    //             if let Some(mut sender) = RUNTIME_SENDER.lock().unwrap().clone() {
    //                 let val = Value::String(text);
    //                 std::thread::spawn(move || {
    //                     futures::executor::block_on(async move {
    //                         sender.send((callback_id, val)).await.ok();
    //                     });
    //                 });
    //             }
    //         }
    //         Task::none()
    //     }
    //     Message::KeyPressed(key, modifiers) => {
    //         use iced::keyboard::key::Named;

    //         if let iced::keyboard::Key::Named(named_key) = key {
    //             match named_key {
    //                 Named::ArrowRight => {
    //                     if state.select_next() {
    //                         scroll_to_selection(state)
    //                     } else {
    //                         Task::none()
    //                     }
    //                 }
    //                 Named::ArrowLeft => {
    //                     if state.select_prev() {
    //                         scroll_to_selection(state)
    //                     } else {
    //                         Task::none()
    //                     }
    //                 }
    //                 Named::ArrowUp => {
    //                     if state.select_up() {
    //                         scroll_to_selection(state)
    //                     } else {
    //                         Task::none()
    //                     }
    //                 }
    //                 Named::ArrowDown => {
    //                     if state.select_down() {
    //                         scroll_to_selection(state)
    //                     } else {
    //                         Task::none()
    //                     }
    //                 }
    //                 Named::Enter => {
    //                     let index = if modifiers.is_empty() {
    //                         Some(0)
    //                     } else if modifiers == Modifiers::COMMAND {
    //                         Some(1)
    //                     } else {
    //                         None
    //                     };

    //                     if let Some(i) = index {
    //                         let action_to_fire = state
    //                             .selected_actions
    //                             .iter()
    //                             .flat_map(|item| match item {
    //                                 ActionPanelItem::Action(action) => {
    //                                     std::slice::from_ref(action).iter()
    //                                 }
    //                                 ActionPanelItem::Section(section) => section.children.iter(),
    //                             })
    //                             .nth(i);

    //                         if let Some(action) = action_to_fire {
    //                             fire_action(&action);
    //                         }
    //                     }
    //                     Task::none()
    //                 }
    //                 Named::Escape => {
    //                     if state.action_panel_visible {
    //                         state.action_panel_visible = false;
    //                         return Task::none();
    //                     }

    //                     if let Some(runtime) = globals::RUNTIME.lock().unwrap().as_mut() {
    //                         let _ = runtime.send_request(&types::SidecarRequest::Pop);
    //                     }
    //                     Task::none()
    //                 }
    //                 _ => Task::none(),
    //             }
    //         } else {
    //             if modifiers == Modifiers::COMMAND {
    //                 if let iced::keyboard::Key::Character(c) = key {
    //                     if c == "k" {
    //                         state.action_panel_visible = !state.action_panel_visible;
    //                     }
    //                 }
    //             }
    //             Task::none()
    //         }
    //     }
    //         }
    //         Task::none()
    //     }
    //     Message::InvokeAction(callback_id) => {
    //         if let Some(mut sender) = RUNTIME_SENDER.lock().unwrap().clone() {
    //             std::thread::spawn(move || {
    //                 futures::executor::block_on(async move {
    //                     sender.send((callback_id, Value::Null)).await.ok();
    //                 });
    //             });
    //         }
    //         Task::none()
    //     }
    //     Message::ToggleActionPanel(visibility) => {
    //         state.action_panel_visible = visibility;
    //         Task::none()
    //     }
    //     Message::Scrolled(viewport) => {
    //         state.viewport = Some(viewport);
    //         Task::none()
    //     }
    //     Message::ImageLoaded(url, handle) => {
    //         image_cache::set(url, handle);
    //         Task::none()
    //     }
    //     Message::LinkClicked(url) => {
    //         println!("link clicked: {}", url);
    //         Task::none()
    //     }
    // }
    Task::none()
}

// fn fire_action(action: &components::types::Action) {
//     if let Some(callback) = &action.on_action {
//         if let Some(mut sender) = RUNTIME_SENDER.lock().unwrap().clone() {
//             let callback_id = callback.id.clone();
//             std::thread::spawn(move || {
//                 futures::executor::block_on(async move {
//                     sender.send((callback_id, Value::Null)).await.ok();
//                 });
//             });
//         }
//     }
// }

// fn scroll_to_selection(state: &State) -> Task<Message> {
//     if let Some(container_index) = state.get_selection_container_index() {
//         if let Ok(cache) = LAYOUT_CACHE.lock() {
//             if let Some(target_bounds) = cache.get(&container_index) {
//                 if let Some(viewport) = &state.viewport {
//                     let view_top = viewport.absolute_offset().y;
//                     let view_bottom = view_top + viewport.bounds().height;
//                     let target_top = target_bounds.y;
//                     let target_bottom = target_top + target_bounds.height;

//                     let new_offset = if target_top < view_top {
//                         Some(scrollable::AbsoluteOffset {
//                             x: 0.0,
//                             y: target_top,
//                         })
//                     } else if target_bottom > view_bottom {
//                         let y = target_bottom - viewport.bounds().height;
//                         Some(scrollable::AbsoluteOffset { x: 0.0, y })
//                     } else {
//                         None
//                     };

//                     if let Some(offset) = new_offset {
//                         return operation::scroll_to(SCROLLABLE.clone(), offset);
//                     }
//                 } else {
//                     return operation::scroll_to(
//                         SCROLLABLE.clone(),
//                         scrollable::AbsoluteOffset {
//                             x: 0.0,
//                             y: target_bounds.y,
//                         },
//                     );
//                 }
//             }
//         }
//     }
//     Task::none()
// }

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
