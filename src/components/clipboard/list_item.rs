use iced::gradient::Linear;
use iced::widget::{container, mouse_area, row, space, stack, text};
use iced::{Alignment, Background, Color, Element, Length};

use crate::clipboard_history::{ClipboardContent, ClipboardEntry};
use crate::screens::clipboard_history::ClipboardHistoryMessage;
use crate::theme::Theme;

pub fn render_list_item<'a>(
    entry: &'a ClipboardEntry,
    index: usize,
    is_selected: bool,
    is_hovered: bool,
    theme: &'a Theme,
) -> Element<'a, ClipboardHistoryMessage> {
    let text_color = theme.colors.text;
    let secondary_text_color = theme.colors.text_60;
    let selection_color = theme.colors.selection;
    let hover_color = Color::from_rgb8(39, 39, 39);

    let (icon, preview_text, is_truncated) = match &entry.content {
        ClipboardContent::Text(t) => {
            let trimmed = t.trim();
            let mut lines = trimmed.lines();
            let first_line = lines.next().unwrap_or("");
            let has_multiline = lines.next().is_some();

            let max_chars = 100;
            let char_count = first_line.chars().count();

            let (display_text, text_was_cut) = if char_count > max_chars {
                (first_line.chars().take(max_chars).collect::<String>(), true)
            } else {
                (first_line.to_string(), false)
            };

            let display_text = if display_text.is_empty() {
                "Empty text".to_string()
            } else {
                display_text
            };

            ("📄", display_text, has_multiline || text_was_cut)
        }
        ClipboardContent::Image(_) => ("🖼️", "Image".to_string(), false),
    };

    let text_widget = text(preview_text)
        .wrapping(text::Wrapping::None)
        .size(14)
        .color(if is_selected {
            text_color
        } else {
            secondary_text_color
        })
        .width(Length::Fill)
        .shaping(iced::widget::text::Shaping::Advanced);

    let content_element: Element<ClipboardHistoryMessage> = if is_truncated {
        let fade_to_color = if is_selected {
            selection_color
        } else if is_hovered {
            hover_color
        } else {
            theme.colors.background
        };

        stack![
            row![text(icon).size(16), text_widget]
                .spacing(10)
                .clip(true)
                .align_y(Alignment::Center),
            row![
                space().width(Length::Fill),
                container(space())
                    .width(100)
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(Background::Gradient(iced::Gradient::Linear(
                            Linear::new(1.571)
                                .add_stop(0.0, Color::TRANSPARENT)
                                .add_stop(1.0, fade_to_color)
                        ))),
                        ..Default::default()
                    })
            ]
        ]
        .into()
    } else {
        row![text(icon).size(16), text_widget]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
    };

    let container_style = move |_theme: &iced::Theme| container::Style {
        background: Some(if is_selected {
            selection_color.into()
        } else if is_hovered {
            hover_color.into()
        } else {
            Color::TRANSPARENT.into()
        }),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let item_container = container(content_element)
        .width(Length::Fill)
        .padding(8)
        .style(container_style);

    mouse_area(item_container)
        .on_press(ClipboardHistoryMessage::ItemClicked(index))
        .on_enter(ClipboardHistoryMessage::ItemHovered(index))
        .on_exit(ClipboardHistoryMessage::ItemUnhovered(index))
        .into()
}
