use iced::Element;
use iced::widget::{checkbox, column, pick_list, radio, row, scrollable, text, text_input};
use serde_json::Value;

use crate::extensions::{Extension, Preference, PreferenceType};
use crate::preferences::{FlareSettings, PreferenceStore};

use super::SettingsMessage;

pub fn settings_view<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
    flare_settings: &'a FlareSettings,
) -> Element<'a, SettingsMessage> {
    let mut content = column![].spacing(20).padding(20);

    content = content.push(text("Settings").size(24));

    content = content.push(render_flare_settings(flare_settings));

    content = content.push(render_extension_settings(extensions, preferences));

    scrollable(content).into()
}

fn render_flare_settings(settings: &FlareSettings) -> Element<'_, SettingsMessage> {
    let mut content = column![].spacing(10);

    content = content.push(text("Flare Settings").size(18));

    let layer_shell_row = row![
        text("Window Mode").width(150),
        radio(
            "Layer Shell (Wayland)",
            true,
            Some(settings.use_layer_shell),
            |v| SettingsMessage::FlareSettingChanged { use_layer_shell: v }
        ),
        radio(
            "Regular Window",
            false,
            Some(settings.use_layer_shell),
            |v| SettingsMessage::FlareSettingChanged { use_layer_shell: v }
        ),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    content = content.push(layer_shell_row);
    content =
        content.push(text("After changing this setting, please restart the application.").size(12));

    content.into()
}

fn render_extension_settings<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
) -> Element<'a, SettingsMessage> {
    let mut content = column![].spacing(10);

    content = content.push(text("Extension Settings").size(18));

    for ext in extensions {
        content = content.push(text(format!("Extension: {}", ext.manifest.title)).size(16));

        if let Some(prefs) = &ext.manifest.preferences {
            for pref in prefs {
                content = content.push(render_preference(
                    &ext.manifest.name,
                    pref,
                    preferences,
                    extensions,
                ));
            }
        }

        for cmd in &ext.manifest.commands {
            if let Some(prefs) = &cmd.preferences {
                if !prefs.is_empty() {
                    content = content.push(text(format!("Command: {}", cmd.title)).size(14));
                    for pref in prefs {
                        content = content.push(render_preference(
                            &ext.manifest.name,
                            pref,
                            preferences,
                            extensions,
                        ));
                    }
                }
            }
        }
    }

    content.into()
}

fn render_preference<'a>(
    extension_id: &'a str,
    pref: &'a Preference,
    store: &'a PreferenceStore,
    extensions: &'a [Extension],
) -> Element<'a, SettingsMessage> {
    let current_value = store.get_value(extension_id, &pref.name, extensions);
    let ext_id = extension_id.to_string();
    let pref_name = pref.name.clone();

    let title = pref.title.as_deref().unwrap_or(&pref.name);

    let input: Element<'a, SettingsMessage> = match pref.preference_type {
        PreferenceType::Textfield | PreferenceType::Password => {
            let value = current_value
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();

            let is_secure = pref.preference_type == PreferenceType::Password;
            let placeholder = pref.placeholder.as_deref().unwrap_or("");

            let ext_id_clone = ext_id.clone();
            let pref_name_clone = pref_name.clone();

            text_input(placeholder, &value)
                .secure(is_secure)
                .on_input(move |v| SettingsMessage::PreferenceChanged {
                    extension_id: ext_id_clone.clone(),
                    key: pref_name_clone.clone(),
                    value: Value::String(v),
                })
                .into()
        }
        PreferenceType::Checkbox => {
            let checked = current_value.and_then(|v| v.as_bool()).unwrap_or(false);
            let label = pref.label.as_deref().unwrap_or("");

            checkbox(label, checked)
                .on_toggle(move |v| SettingsMessage::PreferenceChanged {
                    extension_id: ext_id.clone(),
                    key: pref_name.clone(),
                    value: Value::Bool(v),
                })
                .into()
        }
        PreferenceType::Dropdown => {
            let options: Vec<String> = pref
                .data
                .as_ref()
                .map(|d| d.iter().map(|item| item.title.clone()).collect())
                .unwrap_or_default();

            let selected = current_value
                .and_then(|v| v.as_str().map(String::from))
                .and_then(|val| {
                    pref.data.as_ref().and_then(|d| {
                        d.iter()
                            .find(|item| item.value == val)
                            .map(|item| item.title.clone())
                    })
                });

            let data = pref.data.clone();
            pick_list(options, selected, move |title| {
                let value = data
                    .as_ref()
                    .and_then(|d| d.iter().find(|item| item.title == title))
                    .map(|item| Value::String(item.value.clone()))
                    .unwrap_or(Value::Null);

                SettingsMessage::PreferenceChanged {
                    extension_id: ext_id.clone(),
                    key: pref_name.clone(),
                    value,
                }
            })
            .into()
        }
        _ => text("Unsupported preference type").into(),
    };

    row![text(title).width(150), input]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .into()
}
