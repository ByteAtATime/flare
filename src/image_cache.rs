use std::collections::HashMap;
use std::sync::RwLock;

static IMAGE_CACHE: RwLock<Option<HashMap<String, Vec<u8>>>> = RwLock::new(None);

fn ensure_cache() {
    let mut cache = IMAGE_CACHE.write().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
}

pub fn get(url: &str) -> Option<Vec<u8>> {
    ensure_cache();
    let cache = IMAGE_CACHE.read().unwrap();
    cache.as_ref()?.get(url).cloned()
}

pub fn set(url: String, data: Vec<u8>) {
    ensure_cache();
    let mut cache = IMAGE_CACHE.write().unwrap();
    if let Some(ref mut map) = *cache {
        map.insert(url, data);
    }
}

pub fn fetch_and_cache(url: String) -> Result<Vec<u8>, String> {
    if let Some(cached) = get(&url) {
        return Ok(cached);
    }

    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("Failed to fetch image: {}", e))?;

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read image bytes: {}", e))?
        .to_vec();

    set(url, bytes.clone());
    Ok(bytes)
}
