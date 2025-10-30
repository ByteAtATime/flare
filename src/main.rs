use iced::alignment::Vertical;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Font, Length, Padding, Theme, border};
use iced::{Subscription, futures};
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

fn main() -> Result<(), rustyscript::Error> {
    let (sender, receiver) = mpsc::unbounded();
    *SENDER.lock().unwrap() = Some(sender);
    *RECEIVER.lock().unwrap() = Some(receiver);

    let mut runtime = Runtime::new(RuntimeOptions {
        ..Default::default()
    })?;

    let renderer_module = Module::new("renderer.js", include_str!("../renderer/dist/index.js"));
    runtime.load_module(&renderer_module)?;

    let module = Module::new(
        "setup.js",
        "
        import { createRequire } from 'module';
        const nodeRequire = createRequire(import.meta.url);

        import { raycastApi } from './renderer.js';

        globalThis.require = (moduleName) => {
            if (moduleName === '@raycast/api') {
                return raycastApi;
            }
            return nodeRequire(moduleName);
        };
        
        globalThis.module = { exports: {} };
        ",
    );

    let module2 = Module::new("plugin.js", include_str!("../test/plugin.js"));

    let command_runner = Module::new(
        "runner.js",
        r#"
        await module.exports.default();
    "#,
    );

    runtime.register_async_function("showToast", |args| {
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
    })?;

    runtime.load_module(&module)?;
    runtime.load_module(&module2)?;

    let tokio_runtime = runtime.tokio_runtime();

    tokio_runtime.block_on(async { runtime.load_module_async(&command_runner).await })?;

    iced::application("flare", update, view)
        .subscription(subscription)
        .font(include_bytes!("./assets/Inter.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| rustyscript::Error::Runtime(e.to_string()))
}
