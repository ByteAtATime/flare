use iced::Element;
use iced::widget::{column, row, scrollable, text};

use crate::extensions::{Extension, Preference, PreferenceType};
use crate::message::Message;

pub fn settings_view(extensions: &[Extension]) -> Element<'_, Message> {
    let mut content = column![];

    content = content.push(text("Settings"));

    for ext in extensions {
        content = content.push(text(format!("Extension: {}", ext.manifest.title)));

        if let Some(prefs) = &ext.manifest.preferences {
            content = content.push(text("Extension Preferences:"));
            for pref in prefs {
                content = content.push(render_preference(pref));
            }
        }

        for cmd in &ext.manifest.commands {
            content = content.push(text(format!("Command: {}", cmd.title)));
            if let Some(prefs) = &cmd.preferences {
                for pref in prefs {
                    content = content.push(render_preference(pref));
                }
            }
        }
    }

    scrollable(content).into()
}

fn render_preference(pref: &Preference) -> Element<'_, Message> {
    let type_str = match pref.preference_type {
        PreferenceType::Textfield => "textfield",
        PreferenceType::Password => "password",
        PreferenceType::Checkbox => "checkbox",
        PreferenceType::Dropdown => "dropdown",
        PreferenceType::AppPicker => "apppicker",
        PreferenceType::File => "file",
        PreferenceType::Directory => "directory",
    };

    let required_str = if pref.required {
        "required"
    } else {
        "optional"
    };

    row![
        text(&pref.name),
        text(type_str),
        text(required_str),
        text(&pref.description),
    ]
    .spacing(5)
    .into()
}
