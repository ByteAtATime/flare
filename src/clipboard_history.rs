use crate::encryption;
use arboard::ImageData;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, Once};
use std::thread;
use std::time::Duration;

const MAX_HISTORY_SIZE: usize = 100;
const POLL_INTERVAL_MS: u64 = 500;
const HISTORY_FILE: &str = "clipboard_history.json";

static CLIPBOARD_HISTORY: LazyLock<Mutex<VecDeque<ClipboardEntry>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

static LAST_CONTENT: LazyLock<Mutex<LastContent>> = LazyLock::new(|| Mutex::new(LastContent::None));

static INIT: Once = Once::new();

#[derive(Debug, Clone, PartialEq)]
enum LastContent {
    None,
    Text(String),
    Image(Vec<u8>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardImage {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

impl ClipboardImage {
    fn from_image_data(data: ImageData<'_>) -> Self {
        Self {
            width: data.width,
            height: data.height,
            bytes: data.bytes.into_owned(),
        }
    }

    pub fn to_image_data(&self) -> ImageData<'static> {
        ImageData {
            width: self.width,
            height: self.height,
            bytes: Cow::Owned(self.bytes.clone()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClipboardContent {
    Text(String),
    Image(ClipboardImage),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardEntry {
    pub content: ClipboardContent,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EncryptedEntry {
    encrypted_content: String,
    timestamp: u64,
}

impl ClipboardEntry {
    fn encrypt(&self) -> Result<EncryptedEntry, String> {
        let serialized = serde_json::to_string(&self.content).map_err(|e| e.to_string())?;
        Ok(EncryptedEntry {
            encrypted_content: encryption::encrypt(&serialized)?,
            timestamp: self.timestamp,
        })
    }
}

impl EncryptedEntry {
    fn decrypt(&self) -> Result<ClipboardEntry, String> {
        let decrypted = encryption::decrypt(&self.encrypted_content)?;
        let content: ClipboardContent =
            serde_json::from_str(&decrypted).map_err(|e| e.to_string())?;
        Ok(ClipboardEntry {
            content,
            timestamp: self.timestamp,
        })
    }
}

fn get_history_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("flare").join(HISTORY_FILE);
    }
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".flare").join(HISTORY_FILE);
    }
    PathBuf::from(HISTORY_FILE)
}

fn load_history() -> VecDeque<ClipboardEntry> {
    let path = get_history_path();
    let Ok(data) = fs::read_to_string(&path) else {
        return VecDeque::new();
    };
    let Ok(encrypted_entries) = serde_json::from_str::<Vec<EncryptedEntry>>(&data) else {
        return VecDeque::new();
    };
    encrypted_entries
        .into_iter()
        .filter_map(|e| e.decrypt().ok())
        .collect()
}

fn save_history(history: &VecDeque<ClipboardEntry>) {
    let path = get_history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let encrypted: Vec<_> = history.iter().filter_map(|e| e.encrypt().ok()).collect();
    if let Ok(data) = serde_json::to_string(&encrypted) {
        let _ = fs::write(&path, data);
    }
}

fn add_entry(content: ClipboardContent) {
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();

    if let Some(front) = history.front() {
        match (&front.content, &content) {
            (ClipboardContent::Text(a), ClipboardContent::Text(b)) if a == b => return,
            (ClipboardContent::Image(a), ClipboardContent::Image(b))
                if a.width == b.width && a.height == b.height && a.bytes == b.bytes =>
            {
                return;
            }
            _ => {}
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entry = ClipboardEntry { content, timestamp };

    history.push_front(entry);

    while history.len() > MAX_HISTORY_SIZE {
        history.pop_back();
    }

    save_history(&history);
}

fn poll_clipboard() {
    loop {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    let mut last = LAST_CONTENT.lock().unwrap();
                    if *last != LastContent::Text(text.clone()) {
                        *last = LastContent::Text(text.clone());
                        drop(last);
                        add_entry(ClipboardContent::Text(text));
                    }
                }
            } else if let Ok(image) = clipboard.get_image() {
                let bytes = image.bytes.to_vec();
                let mut last = LAST_CONTENT.lock().unwrap();
                if *last != LastContent::Image(bytes.clone()) {
                    *last = LastContent::Image(bytes);
                    drop(last);
                    add_entry(ClipboardContent::Image(ClipboardImage::from_image_data(
                        image,
                    )));
                }
            }
        }

        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

pub fn init() {
    INIT.call_once(|| {
        let loaded = load_history();
        {
            let mut history = CLIPBOARD_HISTORY.lock().unwrap();
            *history = loaded;
        }

        if let Some(entry) = CLIPBOARD_HISTORY.lock().unwrap().front() {
            let mut last = LAST_CONTENT.lock().unwrap();
            *last = match &entry.content {
                ClipboardContent::Text(t) => LastContent::Text(t.clone()),
                ClipboardContent::Image(img) => LastContent::Image(img.bytes.clone()),
            };
        }

        thread::spawn(poll_clipboard);
    });
}

pub fn get_history() -> Vec<ClipboardEntry> {
    let history = CLIPBOARD_HISTORY.lock().unwrap();
    history.iter().cloned().collect()
}

pub fn clear_history() {
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();
    history.clear();
    save_history(&history);
}

pub fn remove_entry(index: usize) {
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();
    if index < history.len() {
        history.remove(index);
        save_history(&history);
    }
}
