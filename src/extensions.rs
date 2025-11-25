use std::fs;
use std::path::PathBuf;

pub fn get_extensions_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("flare").join("extensions"))
}

pub fn scan_extensions() {
    let Some(extensions_dir) = get_extensions_dir() else {
        eprintln!("Couldn't find a valid path to extensions");
        return;
    };

    if !extensions_dir.exists() {
        eprintln!("Extensions directory does not exist: {:?}", extensions_dir);
        return;
    }

    match fs::read_dir(&extensions_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let package_json = path.join("package.json");
                        if package_json.exists() {
                            eprintln!("ooo extension: {:?}", path.file_name().unwrap_or_default());
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading extensions: {}", e);
        }
    }
}
