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

static LAST_CONTENT: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

static INIT: Once = Once::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: u64,
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
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(entries) = serde_json::from_str::<Vec<ClipboardEntry>>(&data) {
            return VecDeque::from(entries);
        }
    }
    VecDeque::new()
}

fn save_history(history: &VecDeque<ClipboardEntry>) {
    let path = get_history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let entries: Vec<_> = history.iter().collect();
    if let Ok(data) = serde_json::to_string(&entries) {
        let _ = fs::write(&path, data);
    }
}

fn add_entry(content: String) {
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();

    if let Some(front) = history.front() {
        if front.content == content {
            return;
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
        let result = arboard::Clipboard::new().and_then(|mut c| c.get_text());

        if let Ok(text) = result {
            if !text.is_empty() {
                let mut last = LAST_CONTENT.lock().unwrap();
                if *last != text {
                    *last = text.clone();
                    drop(last);
                    add_entry(text);
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
            *last = entry.content.clone();
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
