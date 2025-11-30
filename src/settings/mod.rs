mod app;
mod extensions;
mod view;

pub use app::run;

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Extensions,
}

#[derive(Clone, Debug)]
pub enum SettingsMessage {
    TabChanged(SettingsTab),
    ExtensionSelected(usize),
    PreferenceChanged {
        extension_id: String,
        key: String,
        value: Value,
    },
    FlareSettingChanged {
        use_layer_shell: bool,
    },
}

fn handle_message(
    message: &SettingsMessage,
    preferences: &mut crate::preferences::PreferenceStore,
    flare_settings: &mut crate::preferences::FlareSettings,
) {
    match message {
        SettingsMessage::TabChanged(_) => {}
        SettingsMessage::ExtensionSelected(_) => {}
        SettingsMessage::PreferenceChanged {
            extension_id,
            key,
            value,
        } => {
            preferences.set_value(extension_id, key, value.clone());
            if let Err(e) = preferences.save() {
                eprintln!("Failed to save preferences: {}", e);
            }
        }
        SettingsMessage::FlareSettingChanged { use_layer_shell } => {
            flare_settings.use_layer_shell = *use_layer_shell;
            if let Err(e) = flare_settings.save() {
                eprintln!("Failed to save Flare settings: {}", e);
            }
        }
    }
}
