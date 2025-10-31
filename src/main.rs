use iced::futures;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{column, container, text};
use iced::{Color, Element, Font, Length, Subscription, Theme};
use rustyscript::deno_core::PollEventLoopOptions;
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};
use std::sync::Mutex;

static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);

const INTER_FONT: Font = Font::with_name("Inter");

#[derive(serde::Deserialize)]
struct ToastOptions {
    title: String,
    message: Option<String>,
    style: Option<String>,
}

#[derive(Default)]
struct State {
    toast_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    UpdateToast(String),
}

fn view(state: &State) -> Element<'_, Message> {
    container(column![
        column![].height(Length::Fill),
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
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    struct ToastListener;

    if let Some(receiver) = RECEIVER.lock().unwrap().take() {
        let stream = futures::stream::unfold(receiver, |mut receiver| async {
            receiver.next().await.map(|message| (message, receiver))
        });

        Subscription::run_with_id(std::any::TypeId::of::<ToastListener>(), stream)
    } else {
        Subscription::none()
    }
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
                    println!("ooh new update: {:?}", args[0]);
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
