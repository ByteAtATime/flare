#[cfg(target_os = "linux")]
use std::{
    fs::{File, create_dir_all},
    io::Write,
    process::Command,
};

#[cfg(windows)]
use windows_registry::{CLASSES_ROOT, CURRENT_USER, LOCAL_MACHINE};

pub const SCHEMES: &[&str] = &["flare", "raycast"];

pub fn get_current() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let arg = &args[1];

        if arg.contains("://") {
            return Some(arg.clone());
        }
    }
    None
}

pub fn is_oauth_redirect(url: &str) -> bool {
    url.starts_with("raycast://oauth") || url.starts_with("flare://oauth")
}

pub fn parse_extension_link(url: &str) -> Option<(&str, &str, &str)> {
    let url = url
        .strip_prefix("raycast://extensions/")
        .or_else(|| url.strip_prefix("flare://extensions/"))?;

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 3 {
        Some((
            parts[0],
            parts[1],
            parts[2].split('?').next().unwrap_or(parts[2]),
        ))
    } else {
        None
    }
}

pub fn is_confetti_link(url: &str) -> bool {
    matches!(
        parse_extension_link(url),
        Some(("raycast", "raycast", "confetti"))
    )
}

pub fn handle_oauth_redirect(url: &str) -> bool {
    if !is_oauth_redirect(url) {
        return false;
    }

    let url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let params: std::collections::HashMap<_, _> = url.query_pairs().collect();

    let state = params.get("state").map(|s| s.to_string());
    let code = params.get("code").map(|c| c.to_string());

    if let (Some(state), Some(code)) = (state, code) {
        return crate::oauth::complete(&state, &code);
    }

    false
}

pub fn register_all() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        let current_exe = std::env::current_exe()?;
        let exe = dunce::simplified(&current_exe).display().to_string();

        for scheme in SCHEMES {
            let key_base = format!("Software\\Classes\\{scheme}");

            let key_reg = CURRENT_USER.create(&key_base)?;
            key_reg.set_string("", format!("URL:{} protocol", scheme))?;
            key_reg.set_string("URL Protocol", "")?;

            let icon_reg = CURRENT_USER.create(format!("{key_base}\\DefaultIcon"))?;
            icon_reg.set_string("", format!("{exe},0"))?;

            let cmd_reg = CURRENT_USER.create(format!("{key_base}\\shell\\open\\command"))?;
            cmd_reg.set_string("", format!("\"{exe}\" \"%1\""))?;
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let current_exe = std::env::current_exe()?;
        let file_name = "flare-handler.desktop";

        let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
        let applications_dir = data_dir.join("applications");
        create_dir_all(&applications_dir)?;

        let target_file = applications_dir.join(file_name);

        let mime_types = SCHEMES
            .iter()
            .map(|s| format!("x-scheme-handler/{}", s))
            .collect::<Vec<_>>()
            .join(";");

        let qualified_exec = format!("{} %u", current_exe.display());

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Flare Handler\n\
             Exec={qualified_exec}\n\
             Terminal=false\n\
             MimeType={mime_types};\n\
             NoDisplay=true\n",
            qualified_exec = qualified_exec,
            mime_types = mime_types
        );

        let mut file = File::create(&target_file)?;
        file.write_all(desktop_content.as_bytes())?;

        let _ = Command::new("update-desktop-database")
            .arg(&applications_dir)
            .status();

        for scheme in SCHEMES {
            let mime_type = format!("x-scheme-handler/{}", scheme);
            let _ = Command::new("xdg-mime")
                .args(["default", file_name, &mime_type])
                .status();
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        Ok(())
    }
}

pub fn unregister_all() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        for scheme in SCHEMES {
            let path = format!("Software\\Classes\\{scheme}");
            if CURRENT_USER.open(&path).is_ok() {
                CURRENT_USER.remove_tree(&path)?;
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let file_name = "flare-handler.desktop";
        if let Some(data_dir) = dirs::data_dir() {
            let target_file = data_dir.join("applications").join(file_name);
            if target_file.exists() {
                std::fs::remove_file(target_file)?;
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        Ok(())
    }
}
