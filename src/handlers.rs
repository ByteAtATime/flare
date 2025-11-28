use serde_json::Value;

use crate::globals;
use crate::message::Message;
use crate::types::{ClipboardContent, Tree};

pub type HandlerResult = Result<Option<Value>, String>;

fn ok_none() -> HandlerResult {
    Ok(None)
}

fn ok<T: serde::Serialize>(v: T) -> HandlerResult {
    Ok(Some(serde_json::to_value(v).unwrap()))
}

fn ok_val(v: Value) -> HandlerResult {
    Ok(Some(v))
}

fn send_ui(msg: Message) -> HandlerResult {
    if let Some(sender) = globals::SENDER.lock().unwrap().as_mut() {
        let _ = sender.unbounded_send(msg);
    }
    ok_none()
}

pub mod ui {
    use super::*;

    pub fn show_toast(title: String) -> HandlerResult {
        send_ui(Message::ShowToast(title))
    }

    pub fn update_tree(tree: Tree) -> HandlerResult {
        send_ui(Message::UpdateTree(tree))
    }

    pub fn pop() -> HandlerResult {
        send_ui(Message::PopToRoot)
    }

    pub fn open_settings() -> HandlerResult {
        send_ui(Message::OpenSettings)
    }

    pub fn open_url(url: String) -> HandlerResult {
        send_ui(Message::OpenUrl(url))
    }
}

pub mod cache {
    use super::*;

    pub fn set(n: String, k: String, d: String) -> HandlerResult {
        crate::cache::set(&n, &k, &d).map(|_| None)
    }

    pub fn get(n: String, k: String) -> HandlerResult {
        ok_val(
            crate::cache::get(&n, &k)
                .map(Value::String)
                .unwrap_or(Value::Null),
        )
    }

    pub fn has(n: String, k: String) -> HandlerResult {
        ok(crate::cache::has(&n, &k))
    }

    pub fn remove(n: String, k: String) -> HandlerResult {
        ok(crate::cache::remove(&n, &k))
    }

    pub fn clear(n: String) -> HandlerResult {
        crate::cache::clear(&n).map(|_| None)
    }

    pub fn is_empty(n: String) -> HandlerResult {
        ok(crate::cache::is_empty(&n))
    }
}

pub mod storage {
    use super::*;

    pub fn set(n: String, k: String, d: String) -> HandlerResult {
        crate::storage::set(&n, &k, &d).map(|_| None)
    }

    pub fn get(n: String, k: String) -> HandlerResult {
        ok_val(
            crate::storage::get(&n, &k)
                .map(Value::String)
                .unwrap_or(Value::Null),
        )
    }

    pub fn remove(n: String, k: String) -> HandlerResult {
        ok(crate::storage::remove(&n, &k))
    }

    pub fn clear(n: String) -> HandlerResult {
        crate::storage::clear(&n).map(|_| None)
    }

    pub fn get_all(n: String) -> HandlerResult {
        ok(crate::storage::get_all(&n))
    }
}

pub mod clipboard {
    use super::*;

    fn with_clipboard<F, T>(f: F) -> Result<T, String>
    where
        F: FnOnce(&mut arboard::Clipboard) -> Result<T, String>,
    {
        let mut clipboard_guard = globals::CLIPBOARD.lock().unwrap();
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

    pub fn copy(content: ClipboardContent, _concealed: bool) -> HandlerResult {
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

    pub fn clear() -> HandlerResult {
        with_clipboard(|clipboard| clipboard.clear().map_err(|e| e.to_string())).map(|_| None)
    }

    pub fn read() -> HandlerResult {
        with_clipboard(|clipboard| {
            clipboard.get_text().map_err(|e| e.to_string()).map(|text| {
                let content = crate::types::ClipboardReadResponse {
                    text,
                    html: None,
                    file: None,
                };
                serde_json::to_value(content).unwrap()
            })
        })
        .map(Some)
    }
}

pub mod oauth {
    use crate::transport::Transport;

    use super::*;

    pub fn authorize(id: u32, url: String, state: String, transport: &Transport) {
        globals::OAUTH_PENDING
            .lock()
            .unwrap()
            .insert(state, (id, transport.clone()));

        let _ = crate::utils::open_url(&url);
    }

    pub fn complete(state: &str, code: &str) -> bool {
        let entry = globals::OAUTH_PENDING.lock().unwrap().remove(state);

        if let Some((id, transport)) = entry {
            let response = crate::types::RustResponse::Success {
                id,
                result: Some(serde_json::json!({ "authorizationCode": code })),
            };
            let _ = transport.send(&response);
            true
        } else {
            false
        }
    }
}
