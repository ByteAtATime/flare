use crate::message::Message;
use crate::runtime::SidecarRuntime;
use crate::transport::Transport;
use iced::Rectangle;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
pub static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);
pub static RUNTIME_SENDER: Mutex<Option<mpsc::UnboundedSender<(String, Value)>>> = Mutex::new(None);
pub static RUNTIME: Mutex<Option<SidecarRuntime>> = Mutex::new(None);

pub static IMAGE_LOADER: Mutex<Option<std::sync::mpsc::Sender<String>>> = Mutex::new(None);

pub static POSITION_TRACKER: LazyLock<crate::position::Id> =
    LazyLock::new(|| crate::position::Id::new("items_column"));

pub static LAYOUT_CACHE: LazyLock<Mutex<HashMap<usize, Rectangle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static CLIPBOARD: LazyLock<Mutex<Option<arboard::Clipboard>>> =
    LazyLock::new(|| Mutex::new(None));

pub static OAUTH_PENDING: LazyLock<Mutex<HashMap<String, (u32, Transport)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn send_callback(callback_id: String, value: Value) {
    if let Some(mut sender) = RUNTIME_SENDER.lock().unwrap().clone() {
        std::thread::spawn(move || {
            iced::futures::executor::block_on(async move {
                sender.send((callback_id, value)).await.ok();
            });
        });
    }
}
