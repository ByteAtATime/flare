mod view;

pub use view::settings_view;

use serde_json::Value;

#[derive(Clone, Debug)]
pub enum SettingsMessage {
    PreferenceChanged {
        extension_id: String,
        key: String,
        value: Value,
    },
    FlareSettingChanged {
        use_layer_shell: bool,
    },
}
