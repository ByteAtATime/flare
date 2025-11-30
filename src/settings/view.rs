use iced::widget::{button, column, container, radio, row, rule, scrollable, text};
use iced::{Background, Border, Color, Element, Font, Length};

use crate::extensions::Extension;
use crate::preferences::{FlareSettings, PreferenceStore};
use crate::theme::Theme;

use super::extensions::render_extensions_tab;
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
    selected_extension: Option<usize>,
) -> Element<'a, SettingsMessage> {
    let content: Element<'a, SettingsMessage> = match current_tab {
        SettingsTab::General => render_general_tab(flare_settings, theme),
        SettingsTab::Extensions => {
            render_extensions_tab(extensions, preferences, theme, selected_extension)
        }
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
