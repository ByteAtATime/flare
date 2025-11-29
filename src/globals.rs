use iced::Rectangle;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static POSITION_TRACKER: LazyLock<crate::position::Id> =
    LazyLock::new(|| crate::position::Id::new("items_column"));

pub static LAYOUT_CACHE: LazyLock<Mutex<HashMap<usize, Rectangle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
