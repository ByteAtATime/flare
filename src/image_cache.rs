use iced::widget::image::Handle;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};

static IMAGE_CACHE: RwLock<Option<HashMap<String, Handle>>> = RwLock::new(None);
static PENDING: Mutex<Option<HashSet<String>>> = Mutex::new(None);

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

    ensure_pending();
    let mut pending = PENDING.lock().unwrap();
    if let Some(ref mut set) = *pending {
        set.remove(&url);
    }
}

pub fn should_load(url: &str) -> bool {
    if get(url).is_some() {
        return false;
    }

    ensure_pending();
    let mut pending = PENDING.lock().unwrap();
    let set = pending.as_mut().unwrap();

    if set.contains(url) {
        return false;
    }

    set.insert(url.to_string());
    true
}

pub fn fetch_and_cache(url: String) -> Result<Handle, String> {
    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("Failed to fetch image: {}", e))?;

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read image bytes: {}", e))?
        .to_vec();

    let handle = Handle::from_bytes(bytes);

    set(url, handle.clone());
    Ok(handle)
}

pub fn clear_pending(url: &str) {
    ensure_pending();
    let mut pending = PENDING.lock().unwrap();
    if let Some(ref mut set) = *pending {
        set.remove(url);
    }
}
