use crate::message::Message;
use iced::Rectangle;
use iced::futures::channel::mpsc;
use iced::widget::scrollable;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static SENDER: Mutex<Option<mpsc::UnboundedSender<Message>>> = Mutex::new(None);
pub static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<Message>>> = Mutex::new(None);
pub static CALLBACK_SENDER: Mutex<Option<mpsc::UnboundedSender<String>>> = Mutex::new(None);

pub static SCROLLABLE: LazyLock<scrollable::Id> =
    LazyLock::new(|| scrollable::Id::new("main_scrollable"));
pub static POSITION_TRACKER: LazyLock<crate::position::Id> =
    LazyLock::new(|| crate::position::Id::new("items_column"));

pub static LAYOUT_CACHE: LazyLock<Mutex<HashMap<usize, Rectangle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
