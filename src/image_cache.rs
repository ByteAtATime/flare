use crate::message::Message;
use iced::Task;
use iced::widget::image::Handle;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex, RwLock};

static IMAGE_CACHE: RwLock<Option<HashMap<String, Handle>>> = RwLock::new(None);
static PENDING: Mutex<Option<HashSet<String>>> = Mutex::new(None);
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("flare/0.1.0")
        .build()
        .unwrap()
});

fn ensure_cache() {
    let mut cache = IMAGE_CACHE.write().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
}

fn ensure_pending() {
    let mut pending = PENDING.lock().unwrap();
    if pending.is_none() {
        *pending = Some(HashSet::new());
    }
}

pub fn get(url: &str) -> Option<Handle> {
    ensure_cache();
    let cache = IMAGE_CACHE.read().unwrap();
    cache.as_ref()?.get(url).cloned()
}

pub fn set(url: String, handle: Handle) {
    ensure_cache();

    {
        let mut cache = IMAGE_CACHE.write().unwrap();
        if let Some(ref mut map) = *cache {
            map.insert(url.clone(), handle);
        }
    }

    clear_pending(&url);
}

pub fn clear_pending(url: &str) {
    ensure_pending();
    let mut pending = PENDING.lock().unwrap();
    if let Some(ref mut set) = *pending {
        set.remove(url);
    }
}

/// Returns a Task to fetch the image if it's not already cached or pending.
pub fn fetch(url: String) -> Task<Message> {
    if get(&url).is_some() {
        return Task::none();
    }

    ensure_pending();
    {
        let mut pending = PENDING.lock().unwrap();
        let set = pending.as_mut().unwrap();
        if set.contains(&url) {
            return Task::none();
        }
        set.insert(url.clone());
    }

    Task::perform(download(url.clone()), move |res| match res {
        Ok(handle) => Message::ImageLoaded(url, handle),
        Err(_) => Message::ImageLoadFailed(url),
    })
}

async fn download(url: String) -> Result<Handle, String> {
    let response = CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch image: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    Ok(Handle::from_bytes(bytes))
}
