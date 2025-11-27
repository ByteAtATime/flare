use blake3;
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

fn get_namespace_path(namespace: &str) -> PathBuf {
    let root = get_cache_root();
    let ns = if namespace.is_empty() {
        "default"
    } else {
        namespace
    };
    root.join(ns)
}

fn hash_key_hex(key: &str) -> String {
    let hash = blake3::hash(key.as_bytes());
    hash.to_hex().to_string()
}

fn get_hashed_path(namespace: &str, key: Option<&str>) -> PathBuf {
    let root = get_cache_root();

    let ns = if namespace.is_empty() {
        "default"
    } else {
        namespace
    };
    let ns_path = root.join(ns);

    if let Some(k) = key {
        let safe_key = hash_key_hex(k);
        ns_path.join(safe_key)
    } else {
        ns_path
    }
}

pub fn set(namespace: &str, key: &str, data: &str) -> Result<(), String> {
    let path = get_hashed_path(namespace, Some(key));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&path, data).map_err(|e| e.to_string())?;

    let meta_path = path.with_extension("meta");
    let _ = fs::write(meta_path, key);

    Ok(())
}

pub fn get(namespace: &str, key: &str) -> Option<String> {
    let path = get_hashed_path(namespace, Some(key));
    if path.exists() {
        return fs::read_to_string(path).ok();
    }
    None
}

pub fn has(namespace: &str, key: &str) -> bool {
    let path = get_hashed_path(namespace, Some(key));
    if path.exists() {
        return true;
    }
    false
}

pub fn remove(namespace: &str, key: &str) -> bool {
    let path = get_hashed_path(namespace, Some(key));
    if path.exists() {
        fs::remove_file(&path).is_ok()
    } else {
        false
    }
}

pub fn clear(namespace: &str) -> Result<(), String> {
    let path = get_namespace_path(namespace);
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_empty(namespace: &str) -> bool {
    let path = get_namespace_path(namespace);
    if !path.exists() {
        return true;
    }
    if let Ok(mut entries) = fs::read_dir(path) {
        return entries.next().is_none();
    }
    true
}
