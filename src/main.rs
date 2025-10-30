use iced::Element;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::widget::{button, text};
use iced::{Subscription, futures};
use rustyscript::{Module, Runtime, RuntimeOptions, serde_json::Value};
use std::sync::Mutex;

static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);

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
    button(text(&state.toast_message)).into()
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
                        .send(Message::UpdateToast(
                            value.message.unwrap_or("".to_string()).clone(),
                        ))
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
        .run()
        .map_err(|e| rustyscript::Error::Runtime(e.to_string()))
}
