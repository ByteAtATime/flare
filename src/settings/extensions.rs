use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, text, text_input,
};
use iced::{Background, Border, Color, Element, Length};
use serde_json::Value;

use crate::extensions::{Extension, Preference, PreferenceType};
use crate::preferences::PreferenceStore;
use crate::theme::Theme;

use super::SettingsMessage;

pub fn render_extensions_tab<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
    theme: &'a Theme,
    selected_extension: Option<usize>,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let bg_color = theme.colors.background;

    let mut ext_list = column![].spacing(2);
    for (idx, ext) in extensions.iter().enumerate() {
        let is_selected = selected_extension == Some(idx);
        let item_bg = if is_selected {
            theme.colors.selection
        } else {
            Color::TRANSPARENT
        };

        ext_list = ext_list.push(
            button(text(&ext.manifest.title).color(text_color))
                .on_press(SettingsMessage::ExtensionSelected(idx))
                .width(Length::Fill)
                .style(move |_theme, status| {
                    let bg = match status {
                        button::Status::Hovered if !is_selected => theme.colors.text_10,
                        _ => item_bg,
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        text_color,
                        border: Border::default(),
                        ..Default::default()
                    }
                }),
        );
    }

    let left_panel = container(scrollable(ext_list).height(Length::Fill))
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        });

    let right_panel: Element<'a, SettingsMessage> = if let Some(idx) = selected_extension {
        if let Some(ext) = extensions.get(idx) {
            render_extension_preferences(ext, preferences, extensions, theme)
        } else {
            container(text("Select an extension").color(theme.colors.text_60))
                .center(Length::Fill)
                .into()
        }
    } else {
        container(text("Select an extension").color(theme.colors.text_60))
            .center(Length::Fill)
            .into()
    };

    let right_container = container(right_panel)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        });

    row![
        left_panel,
        rule::vertical(1).style(|iced_theme| rule::Style {
            color: theme.colors.border_10,
            ..rule::default(iced_theme)
        }),
        right_container
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn render_extension_preferences<'a>(
    ext: &'a Extension,
    preferences: &'a PreferenceStore,
    extensions: &'a [Extension],
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let bg_color = theme.colors.background;

    let mut content = column![text(&ext.manifest.title).size(20).color(text_color),].spacing(16);

    if let Some(prefs) = &ext.manifest.preferences {
        for pref in prefs {
            content = content.push(render_preference(
                &ext.manifest.name,
                pref,
                preferences,
                extensions,
                theme,
            ));
        }
    }

    for cmd in &ext.manifest.commands {
        if let Some(prefs) = &cmd.preferences {
            if !prefs.is_empty() {
                content = content.push(
                    text(format!("Command: {}", cmd.title))
                        .size(14)
                        .color(text_color),
                );
                for pref in prefs {
                    content = content.push(render_preference(
                        &ext.manifest.name,
                        pref,
                        preferences,
                        extensions,
                        theme,
                    ));
                }
            }
        }
    }

    scrollable(content)
        .style(move |iced_theme, status| scrollable::Style {
            container: container::Style {
                background: Some(bg_color.into()),
                ..Default::default()
            },
            ..scrollable::default(iced_theme, status)
        })
        .into()
}

fn render_preference<'a>(
    extension_id: &'a str,
    pref: &'a Preference,
    store: &'a PreferenceStore,
    extensions: &'a [Extension],
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
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

    row![text(title).width(150).color(text_color), input]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .into()
}
