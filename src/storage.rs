use crate::encryption;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn get_storage_root() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("flare").join("storage");
    }

    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".flare").join("storage");
    }

    PathBuf::from(".flare_storage")
}

fn get_path(namespace: &str, key: Option<&str>) -> PathBuf {
    let root = get_storage_root();

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

    let encrypted = encryption::encrypt(data)?;
    fs::write(path, encrypted).map_err(|e| e.to_string())
}

pub fn get(namespace: &str, key: &str) -> Option<String> {
    let path = get_path(namespace, Some(key));
    let encrypted = fs::read_to_string(path).ok()?;
    encryption::decrypt(&encrypted).ok()
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

pub fn get_all(namespace: &str) -> HashMap<String, String> {
    let path = get_path(namespace, None);
    let mut result = HashMap::new();

    if !path.exists() {
        return result;
    }

    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            if let Ok(encrypted) = fs::read_to_string(entry.path()) {
                if let Ok(content) = encryption::decrypt(&encrypted) {
                    if let Some(filename) = entry.file_name().to_str() {
                        if let Some(key) = decode_key(filename) {
                            result.insert(key, content);
                        }
                    }
                }
            }
        }
    }

    result
}

fn decode_key(hex: &str) -> Option<String> {
    let bytes: Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect();

    bytes.ok().and_then(|b| String::from_utf8(b).ok())
}
