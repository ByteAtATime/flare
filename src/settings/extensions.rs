use iced::widget::{
    button, checkbox, column, container, image, pick_list, row, rule, scrollable, space, svg, text,
    text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, color};
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
    search_query: &'a str,
) -> Element<'a, SettingsMessage> {
    let text_color = theme.colors.text;
    let bg_color = theme.colors.background;

    let search_input = text_input("Search...", search_query)
        .on_input(SettingsMessage::ExtensionSearchChanged)
        .size(16)
        .width(Length::Fill)
        .padding([4, 10])
        .style(|iced_theme, status| text_input::Style {
            background: theme.colors.background.into(),
            border: Border {
                color: theme.colors.text_20,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..text_input::default(iced_theme, status)
        });

    let search_bar = container(search_input).height(22);

    let left_header = container(search_bar)
        .padding(Padding::from([7, 16])) // remove 1px of vertical padding to compensate for border
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        });

    let right_header: Element<'a, SettingsMessage> = if let Some(idx) = selected_extension {
        if let Some(ext) = extensions.get(idx) {
            let icon_element: Element<'a, SettingsMessage> = if !ext.manifest.icon.is_empty() {
                let icon_path = ext.path.join("assets").join(&ext.manifest.icon);
                if icon_path.extension().map_or(false, |e| e == "svg") {
                    svg(icon_path).width(32).height(32).into()
                } else {
                    image(icon_path).width(32).height(32).into()
                }
            } else {
                container(text("")).width(32).height(32).into()
            };

            container(
                row![
                    icon_element,
                    text(&ext.manifest.title).size(20).color(text_color),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 16]))
            .width(Length::Fixed(350.0))
            .into()
        } else {
            container(text("")).width(Length::Fixed(350.0)).into()
        }
    } else {
        container(text("")).width(Length::Fixed(350.0)).into()
    };

    let header_row = row![
        left_header,
        rule::vertical(1).style(|iced_theme| rule::Style {
            color: theme.colors.border_10,
            ..rule::default(iced_theme)
        }),
        right_header
    ]
    .height(Length::Shrink);

    let table_header = row![text("Name").size(12).color(theme.colors.text_60)]
        .height(Length::Fill)
        .padding([0, 20])
        .align_y(Alignment::Center);
    let table_header_container = container(column![
        rule::horizontal(1).style(|iced_theme| rule::Style {
            color: theme.colors.border_10,
            ..rule::default(iced_theme)
        }),
        table_header,
        rule::horizontal(1).style(|iced_theme| rule::Style {
            color: theme.colors.border_10,
            ..rule::default(iced_theme)
        }),
    ])
    .height(32)
    .style(|_theme| container::Style {
        background: Some(color!(0x2c2c2c).into()), // TODO: this isn't anywhere in the theme
        ..Default::default()
    });

    let mut ext_list = column![].padding([0, 8]);
    for (idx, ext) in extensions.iter().enumerate() {
        let title_lower = ext.manifest.title.to_lowercase();
        let query_lower = search_query.to_lowercase();
        if !search_query.is_empty() && !title_lower.contains(&query_lower) {
            continue;
        }

        let is_selected = selected_extension == Some(idx);

        let icon_element: Element<'a, SettingsMessage> = if !ext.manifest.icon.is_empty() {
            let icon_path = ext.path.join("assets").join(&ext.manifest.icon);
            if icon_path.extension().map_or(false, |e| e == "svg") {
                svg(icon_path).width(16).height(16).into()
            } else {
                image(icon_path).width(16).height(16).into()
            }
        } else {
            container(text("")).width(16).height(16).into()
        };

        let accordion_arrow = space().width(10).height(10);

        ext_list = ext_list.push(
            button(
                row![
                    accordion_arrow,
                    icon_element,
                    text(&ext.manifest.title).color(text_color)
                ]
                .spacing(8)
                .height(Length::Fixed(32.0))
                .align_y(Alignment::Center),
            )
            .on_press(SettingsMessage::ExtensionSelected(idx))
            .width(Length::Fill)
            .padding([0, 12])
            .style(move |_theme, _status| {
                let background_color = if is_selected {
                    Color {
                        a: 0.6,
                        ..theme.colors.blue
                    }
                } else if idx % 2 == 0 {
                    theme.colors.background
                } else {
                    color!(0x272727) // TODO: this isn't anywhere in the theme
                };
                button::Style {
                    background: Some(background_color.into()),
                    text_color,
                    border: Border::default().rounded(6),
                    ..Default::default()
                }
            }),
        );
    }

    let left_content = container(column![
        table_header_container,
        scrollable(ext_list).height(Length::Fill)
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        ..Default::default()
    });

    let right_content: Element<'a, SettingsMessage> = if let Some(idx) = selected_extension {
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

    let right_content_container = container(right_content)
        .width(Length::Fixed(350.0))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        });

    let content_row = row![
        left_content,
        rule::vertical(1).style(|iced_theme| rule::Style {
            color: theme.colors.border_10,
            ..rule::default(iced_theme)
        }),
        right_content_container
    ]
    .height(Length::Fill);

    column![header_row, content_row]
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

    let mut content = column![
        column![
            text("Description").size(12).color(theme.colors.text_60),
            text(&ext.manifest.description).color(text_color),
        ]
        .spacing(4),
    ]
    .spacing(16)
    .padding(16);

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
