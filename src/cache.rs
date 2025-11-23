use std::fs;
use std::path::PathBuf;

fn get_cache_root() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        return cache_dir.join("flare");
    }

    // ideally these fallbacks wouldn't be necessary - when tf would your home dir not exist
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".flare").join("cache");
    }

    PathBuf::from(".flare_cache")
}

fn get_path(namespace: &str, key: Option<&str>) -> PathBuf {
    let root = get_cache_root();

    let ns = if namespace.is_empty() {
        "default"
    } else {
        namespace
    };
    let ns_path = root.join(ns);

    if let Some(k) = key {
        let safe_key: String = k.as_bytes().iter().map(|b| format!("{:02x}", b)).collect();
        ns_path.join(safe_key)
    } else {
        ns_path
    }
}

pub fn set(namespace: &str, key: &str, data: &str) -> Result<(), String> {
    let path = get_path(namespace, Some(key));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn get(namespace: &str, key: &str) -> Option<String> {
    let path = get_path(namespace, Some(key));
    fs::read_to_string(path).ok()
}

pub fn has(namespace: &str, key: &str) -> bool {
    let path = get_path(namespace, Some(key));
    path.exists()
}

pub fn remove(namespace: &str, key: &str) -> bool {
    let path = get_path(namespace, Some(key));
    if path.exists() {
        fs::remove_file(path).is_ok()
    } else {
        false
    }
}

pub fn clear(namespace: &str) -> Result<(), String> {
    let path = get_path(namespace, None);
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_empty(namespace: &str) -> bool {
    let path = get_path(namespace, None);
    if !path.exists() {
        return true;
    }
    if let Ok(mut entries) = fs::read_dir(path) {
        return entries.next().is_none();
    }
    true
}
