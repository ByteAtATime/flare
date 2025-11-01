mod components;
mod types;

use iced::futures;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{column, container, stack};
use iced::{Element, Length, Subscription};
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};
use std::sync::Mutex;

use types::{ToastOptions, Tree};

use crate::components::actions::render_action_panel;
use crate::components::footer::render_footer;

static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);
static CALLBACK_SENDER: Mutex<Option<mpsc::UnboundedSender<String>>> = Mutex::new(None);

fn update_selected_actions(state: &mut State) {
    use components::Component;

    state.selected_actions = state
        .tree
        .as_ref()
        .and_then(|tree| tree.children.first())
        .and_then(|component| {
            if let Component::Grid(grid_props) = component {
                grid_props.sections.first()
            } else {
                None
            }
        })
        .and_then(|section| section.items.get(state.selected_index))
        .and_then(|item| item.actions.as_ref())
        .map(|action_panel| action_panel.children.clone())
        .unwrap_or_default();
}

#[derive(Default)]
struct State {
    toast_message: String,
    tree: Option<Tree>,
    selected_index: usize,
    selected_actions: Vec<components::types::Action>,
    action_panel_visible: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateToast(String),
    UpdateTree(Tree),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    InvokeAction(String),
    CloseActionPanel,
}

fn view(state: &State) -> Element<'_, Message> {
    let content = state
        .tree
        .as_ref()
        .map(|tree| {
            tree.children
                .iter()
                .fold(column![].height(Length::Fill), |col, child| {
                    col.push(components::render_component(child, state.selected_index))
                })
        })
        .unwrap_or_else(|| column![].height(Length::Fill));

    let base = column![content, render_footer(state)];

    container(if state.action_panel_visible {
        stack![base, render_action_panel(state)].into()
    } else {
        Element::from(base)
    })
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateToast(new_message) => state.toast_message = new_message,
        Message::UpdateTree(tree) => {
            println!("Tree update: {:?}", tree);
            state.tree = Some(tree);
            update_selected_actions(state);
        }
        Message::KeyPressed(key, modifiers) => {
            use components::Component;
            use iced::keyboard::key::Named;

            if let iced::keyboard::Key::Named(named_key) = key {
                let total_items = state
                    .tree
                    .as_ref()
                    .and_then(|tree| tree.children.first())
                    .and_then(|component| {
                        if let Component::Grid(grid_props) = component {
                            grid_props.sections.first()
                        } else {
                            None
                        }
                    })
                    .map(|section| section.items.len())
                    .unwrap_or(0);

                if total_items > 0 {
                    match named_key {
                        Named::ArrowRight => {
                            state.selected_index = (state.selected_index + 1) % total_items;
                            update_selected_actions(state);
                        }
                        Named::ArrowLeft => {
                            state.selected_index = if state.selected_index == 0 {
                                total_items - 1
                            } else {
                                state.selected_index - 1
                            };
                            update_selected_actions(state);
                        }
                        Named::Enter => {
                            if let Some(action) = state.selected_actions.get(0) {
                                if let Some(callback) = &action.on_action {
                                    if let Some(mut sender) =
                                        CALLBACK_SENDER.lock().unwrap().clone()
                                    {
                                        let callback_id = callback.id.clone();
                                        std::thread::spawn(move || {
                                            futures::executor::block_on(async move {
                                                sender.send(callback_id).await.ok();
                                            });
                                        });
                                    }
                                }
                            }
                        }
                        Named::Escape => {
                            if state.action_panel_visible {
                                state.action_panel_visible = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if modifiers.contains(iced::keyboard::Modifiers::COMMAND) {
                if let iced::keyboard::Key::Character(c) = key {
                    if c == "k" {
                        state.action_panel_visible = !state.action_panel_visible;
                    }
                }
            }
        }
        Message::InvokeAction(callback_id) => {
            if let Some(mut sender) = CALLBACK_SENDER.lock().unwrap().clone() {
                std::thread::spawn(move || {
                    futures::executor::block_on(async move {
                        sender.send(callback_id).await.ok();
                    });
                });
            }
        }
        Message::CloseActionPanel => {
            state.action_panel_visible = false;
        }
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    struct ToastListener;

    let message_stream = if let Some(receiver) = RECEIVER.lock().unwrap().take() {
        let stream = futures::stream::unfold(receiver, |mut receiver| async {
            receiver.next().await.map(|message| (message, receiver))
        });

        Subscription::run_with_id(std::any::TypeId::of::<ToastListener>(), stream)
    } else {
        Subscription::none()
    };

    let keyboard_sub =
        iced::keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)));

    Subscription::batch(vec![message_stream, keyboard_sub])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::unbounded();
    *SENDER.lock().unwrap() = Some(sender);
    *RECEIVER.lock().unwrap() = Some(receiver);

    let (callback_sender, mut callback_receiver) = mpsc::unbounded::<String>();
    *CALLBACK_SENDER.lock().unwrap() = Some(callback_sender);

    std::thread::spawn(move || {
        let mut runtime = Runtime::new(RuntimeOptions::default()).unwrap();

        let renderer_module = Module::new("renderer.js", include_str!("../renderer/dist/index.js"));
        runtime.load_module(&renderer_module).unwrap();

        let module = Module::new(
            "setup.js",
            "
            import { createRequire } from 'module';
            const nodeRequire = createRequire(import.meta.url);
    
            import { raycastApi, React, ReactJsxRuntime } from './renderer.js';
    
            globalThis.require = (moduleName) => {
                if (moduleName === '@raycast/api') {
                    return raycastApi;
                }
    
                if (moduleName === 'react') return React;
                if (moduleName === 'react/jsx-runtime') return ReactJsxRuntime;
    
                return nodeRequire(moduleName);
            };
            
            globalThis.module = { exports: {} };
            ",
        );
        runtime.load_module(&module).unwrap();

        let module2 = Module::new("plugin.js", include_str!("../test/plugin.js"));
        runtime.load_module(&module2).unwrap();

        let command_runner = Module::new(
            "runner.js",
            r#"
            import { React, updateContainer, invokeCallback } from './renderer.js';
    
            const PluginRoot = module.exports.default;
            const AppElement = React.createElement(PluginRoot);
            updateContainer(AppElement, () => {
                console.log("initial render callback fired!");
            });

            export { invokeCallback };
        "#,
        );

        runtime
            .register_async_function("showToast", |args| {
                Box::pin(async move {
                    if let Ok(value) = serde_json::from_value::<ToastOptions>(args[0].clone()) {
                        if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                            sender
                                .send(Message::UpdateToast(value.title.clone()))
                                .await
                                .unwrap();
                        }
                    }
                    Ok(Value::Null)
                })
            })
            .unwrap();

        runtime
            .register_async_function("updateTree", |args| {
                Box::pin(async move {
                    if let Ok(tree) = serde_json::from_value::<Tree>(args[0].clone()) {
                        if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                            sender.send(Message::UpdateTree(tree)).await.unwrap();
                        }
                    }
                    Ok(Value::Null)
                })
            })
            .unwrap();

        let command_runner_handle = runtime.load_module(&command_runner).unwrap();

        loop {
            match callback_receiver.try_next() {
                Ok(Some(callback_id)) => {
                    let result: Result<Value, _> = runtime.call_function(
                        Some(&command_runner_handle),
                        "invokeCallback",
                        &[Value::String(callback_id)],
                    );
                    if let Err(e) = result {
                        eprintln!("callback died: {:?}", e);
                    }
                }
                Ok(None) => break,
                Err(_) => {}
            }
        }
    });

    iced::application("flare", update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}
