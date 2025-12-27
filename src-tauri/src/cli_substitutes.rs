use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

/// CLI binary substitution registry
/// Maps macOS binary names to their Linux download URLs and extraction paths

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSubstitute {
    /// Name of the binary file to substitute
    pub binary_name: String,
    /// URL template for downloading Linux version (use {arch} placeholder)
    pub download_url_template: String,
    /// Path within the archive to the binary (if in a subdirectory)
    pub binary_path_in_archive: Option<String>,
    /// Whether the download is a tar.gz archive
    pub is_tar_gz: bool,
}

/// Built-in registry of known CLI substitutes
pub fn get_builtin_registry() -> HashMap<String, CliSubstitute> {
    let mut registry = HashMap::new();

    // Speedtest CLI by Ookla
    registry.insert(
        "speedtest".to_string(),
        CliSubstitute {
            binary_name: "speedtest".to_string(),
            download_url_template:
                "https://install.speedtest.net/app/cli/ookla-speedtest-1.2.0-linux-{arch}.tgz"
                    .to_string(),
            binary_path_in_archive: Some("speedtest".to_string()),
            is_tar_gz: true,
        },
    );

    registry
}

/// Get the current architecture string for download URLs
fn get_arch_string() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64"
    }
    #[cfg(target_arch = "arm")]
    {
        "armhf"
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
    {
        "x86_64" // fallback
    }
}

/// Download and extract a Linux CLI binary substitute
pub async fn download_substitute(
    substitute: &CliSubstitute,
    target_dir: &Path,
) -> Result<PathBuf, String> {
    let arch = get_arch_string();
    let url = substitute.download_url_template.replace("{arch}", arch);

    // Download the archive
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to download CLI substitute from {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download CLI substitute: HTTP {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Ensure target directory exists
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let target_binary_path = target_dir.join(&substitute.binary_name);

    if substitute.is_tar_gz {
        // Extract from tar.gz
        let cursor = std::io::Cursor::new(bytes.as_ref());
        let tar = GzDecoder::new(cursor);
        let mut archive = Archive::new(tar);

        let binary_path_in_archive = substitute
            .binary_path_in_archive
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(&substitute.binary_name);

        let entries = archive.entries().map_err(|e| e.to_string())?;
        for entry_result in entries {
            let mut entry = entry_result.map_err(|e| e.to_string())?;
            let entry_path = entry.path().map_err(|e| e.to_string())?;

            // Check if this is the binary we want
            if entry_path.ends_with(binary_path_in_archive) {
                // Extract to target location
                let mut file = fs::File::create(&target_binary_path)
                    .map_err(|e| format!("Failed to create binary file: {}", e))?;
                std::io::copy(&mut entry, &mut file)
                    .map_err(|e| format!("Failed to write binary: {}", e))?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&target_binary_path, fs::Permissions::from_mode(0o755))
                        .map_err(|e| format!("Failed to set permissions: {}", e))?;
                }

                return Ok(target_binary_path);
            }
        }

        Err(format!(
            "Binary '{}' not found in archive",
            binary_path_in_archive
        ))
    } else {
        // Direct binary download
        fs::write(&target_binary_path, &bytes)
            .map_err(|e| format!("Failed to write binary: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_binary_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        Ok(target_binary_path)
    }
}

/// Check if a substitute exists for a given binary name
pub fn find_substitute(binary_name: &str) -> Option<CliSubstitute> {
    get_builtin_registry().get(binary_name).cloned()
}

/// Substitute macOS binaries with Linux equivalents in an extension
pub async fn substitute_macos_binaries(
    extension_dir: &Path,
    macho_binaries: &[String],
) -> Result<Vec<String>, String> {
    let support_cli_dir = extension_dir.join("support").join("cli");
    let assets_dir = extension_dir.join("assets");

    let mut substituted = Vec::new();

    for binary_name in macho_binaries {
        if let Some(substitute) = find_substitute(binary_name) {
            // Download and install the Linux substitute
            match download_substitute(&substitute, &support_cli_dir).await {
                Ok(path) => {
                    // Also check if there's a binary in assets that needs replacing
                    let asset_binary = assets_dir.join(binary_name);
                    if asset_binary.exists() {
                        // Replace the asset binary with a symlink or copy
                        fs::copy(&path, &asset_binary)
                            .map_err(|e| format!("Failed to replace asset binary: {}", e))?;
                    }

                    substituted.push(binary_name.clone());
                    tracing::info!(
                        binary = %binary_name,
                        "Substituted macOS binary with Linux version"
                    );
                }
                Err(e) => {
                    tracing::warn!(binary = %binary_name, error = %e, "Failed to substitute binary");
                }
            }
        }
    }

    Ok(substituted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_speedtest() {
        let registry = get_builtin_registry();
        assert!(registry.contains_key("speedtest"));
    }

    #[test]
    fn test_find_substitute() {
        assert!(find_substitute("speedtest").is_some());
        assert!(find_substitute("nonexistent").is_none());
    }

    #[test]
    fn test_arch_string() {
        let arch = get_arch_string();
        assert!(!arch.is_empty());
    }
}
