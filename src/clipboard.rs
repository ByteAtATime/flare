use crate::types::{ClipboardContent, ClipboardReadResponse};
use std::sync::{LazyLock, Mutex};

static CLIPBOARD: LazyLock<Mutex<Option<arboard::Clipboard>>> = LazyLock::new(|| Mutex::new(None));

fn with_clipboard<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut arboard::Clipboard) -> Result<T, String>,
{
    let mut clipboard_guard = CLIPBOARD.lock().unwrap();
    if clipboard_guard.is_none() {
        match arboard::Clipboard::new() {
            Ok(c) => *clipboard_guard = Some(c),
            Err(e) => return Err(e.to_string()),
        }
    }

    if let Some(clipboard) = clipboard_guard.as_mut() {
        f(clipboard)
    } else {
        Err("Failed to initialize clipboard".to_string())
    }
}

pub fn copy(
    content: ClipboardContent,
    _concealed: bool,
) -> Result<Option<serde_json::Value>, String> {
    with_clipboard(|clipboard| {
        let res = match content {
            ClipboardContent::Text { text } => clipboard.set_text(text),
            ClipboardContent::File { file } => clipboard.set_text(file),
            ClipboardContent::Html { html, text } => {
                if let Some(t) = text {
                    let _ = clipboard.set_text(t);
                }
                clipboard.set_html(html, None)
            }
        };
        res.map_err(|e| e.to_string())
    })
    .map(|_| None)
}

pub fn clear() -> Result<Option<serde_json::Value>, String> {
    with_clipboard(|clipboard| clipboard.clear().map_err(|e| e.to_string())).map(|_| None)
}

pub fn read() -> Result<Option<serde_json::Value>, String> {
    with_clipboard(|clipboard| {
        clipboard.get_text().map_err(|e| e.to_string()).map(|text| {
            let content = ClipboardReadResponse {
                text,
                html: None,
                file: None,
            };
            serde_json::to_value(content).unwrap()
        })
    })
    .map(Some)
}
