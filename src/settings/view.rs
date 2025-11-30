use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, rule, scrollable, text, text_input,
};
use iced::{Background, Border, Color, Element, Font, Length};
use serde_json::Value;

use crate::extensions::{Extension, Preference, PreferenceType};
use crate::preferences::{FlareSettings, PreferenceStore};
use crate::theme::Theme;

use super::{SettingsMessage, SettingsTab};

const ICON_FONT: Font = Font::with_name("Raycast-Icons");

const GEAR_ICON: &str = "\u{e96b}";
const CHIP_ICON: &str = "\u{e970}";

fn tab_button<'a>(
    label: &'a str,
    icon: &'a str,
    tab: SettingsTab,
    current_tab: SettingsTab,
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let is_selected = tab == current_tab;
    let text_color = if is_selected {
        theme.colors.text
    } else {
        theme.colors.text_60
    };
    let bg_color = if is_selected {
        theme.colors.selection
    } else {
        Color::TRANSPARENT
    };

    button(
        column![text(icon).font(ICON_FONT).size(20), text(label).size(11)]
            .spacing(4)
            .align_x(iced::Alignment::Center),
    )
    .on_press(SettingsMessage::TabChanged(tab))
    .padding([8, 16])
    .style(move |_theme, status| {
        let bg = match status {
            button::Status::Hovered if !is_selected => theme.colors.text_40,
            _ => bg_color,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

fn tab_bar<'a>(current_tab: SettingsTab, theme: &'a Theme) -> Element<'a, SettingsMessage> {
    container(
        column![
            row![
                tab_button(
                    "General",
                    GEAR_ICON,
                    SettingsTab::General,
                    current_tab,
                    theme
                ),
                tab_button(
                    "Extensions",
                    CHIP_ICON,
                    SettingsTab::Extensions,
                    current_tab,
                    theme
                ),
            ]
            .spacing(16)
            .align_y(iced::Alignment::Center),
            rule::horizontal(1).style(|iced_theme| rule::Style {
                color: theme.colors.border_10,
                ..rule::default(iced_theme)
            })
        ]
        .align_x(iced::Alignment::Center),
    )
    .padding([12, 20])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(theme.colors.background.into()),
        ..Default::default()
    })
    .into()
}

pub fn settings_view<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
    flare_settings: &'a FlareSettings,
    theme: &'a Theme,
    current_tab: SettingsTab,
) -> Element<'a, SettingsMessage> {
    let content: Element<'a, SettingsMessage> = match current_tab {
        SettingsTab::General => render_general_tab(flare_settings, theme),
        SettingsTab::Extensions => render_extensions_tab(extensions, preferences, theme),
    };

    column![tab_bar(current_tab, theme), content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_general_tab<'a>(
    settings: &'a FlareSettings,
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let bg_color = theme.colors.background;

    let content = column![
        text("General").size(20).color(text_color),
        render_flare_settings(settings, theme),
    ]
    .spacing(20)
    .padding(20);

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

fn render_extensions_tab<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let bg_color = theme.colors.background;

    let content = column![
        text("Extensions").size(20).color(text_color),
        render_extension_settings(extensions, preferences, theme),
    ]
    .spacing(20)
    .padding(20);

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

fn render_flare_settings<'a>(
    settings: &'a FlareSettings,
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let mut content = column![].spacing(10);

    let layer_shell_row = row![
        text("Window Mode").width(150).color(text_color),
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
    content = content.push(
        text("After changing this setting, please restart the application.")
            .size(12)
            .color(theme.colors.text_60),
    );

    content.into()
}

fn render_extension_settings<'a>(
    extensions: &'a [Extension],
    preferences: &'a PreferenceStore,
    theme: &'a Theme,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let mut content = column![].spacing(10);

    for ext in extensions {
        content = content.push(
            text(format!("Extension: {}", ext.manifest.title))
                .size(16)
                .color(text_color),
        );

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
    }

    content.into()
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
