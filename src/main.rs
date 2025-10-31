mod components;
mod types;

use iced::futures;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{column, container, text};
use iced::{Color, Element, Font, Length, Subscription, Theme};
use rustyscript::deno_core::PollEventLoopOptions;
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};
use std::sync::Mutex;

use types::{ToastOptions, Tree};

static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);

const INTER_FONT: Font = Font::with_name("Inter");

#[derive(Default)]
struct State {
    toast_message: String,
    tree: Option<Tree>,
    selected_index: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateToast(String),
    UpdateTree(Tree),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
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

    container(column![
        content,
        container(
            text(&state.toast_message)
                .size(16)
                .font(INTER_FONT)
                .shaping(text::Shaping::Advanced)
        )
        .width(Length::Fill)
        .padding([0, 8])
        .center_y(40)
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::from_rgb8(0x22, 0x22, 0x22).into()),
            text_color: Some(Color::WHITE),
            ..Default::default()
        })
    ])
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateToast(new_message) => state.toast_message = new_message,
        Message::UpdateTree(tree) => {
            println!("Tree update: {:?}", tree);
            state.tree = Some(tree);
        }
        Message::KeyPressed(key, _modifiers) => {
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
                        }
                        Named::ArrowLeft => {
                            state.selected_index = if state.selected_index == 0 {
                                total_items - 1
                            } else {
                                state.selected_index - 1
                            };
                        }
                        _ => {}
                    }
                }
            }
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

    std::thread::spawn(|| {
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
            import { React, updateContainer } from './renderer.js';
    
            const PluginRoot = module.exports.default;
            const AppElement = React.createElement(PluginRoot);
            updateContainer(AppElement, () => {
                console.log("initial render callback fired!");
            });
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

        runtime.load_module(&command_runner).unwrap();

        runtime
            .block_on_event_loop(PollEventLoopOptions::default(), None)
            .unwrap();
    });

    iced::application("flare", update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}
