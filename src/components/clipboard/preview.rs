use chrono::{Local, TimeZone};
use iced::widget::text::LineHeight;
use iced::widget::{column, container, image, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length};

use crate::clipboard_history::{ClipboardContent, ClipboardEntry};
use crate::components::types::parse_hex_color;
use crate::screens::clipboard_history::ClipboardHistoryMessage;
use crate::theme::Theme;

pub fn render_preview_pane<'a>(
    entry: Option<&'a ClipboardEntry>,
    theme: &'a Theme,
    preview_viewport: Option<&'a iced::widget::scrollable::Viewport>,
    preview_scrollable_id: &'a iced::widget::Id,
    has_application_metadata: bool,
) -> Element<'a, ClipboardHistoryMessage> {
    let secondary_text_color = theme.colors.text_60;

    if let Some(entry) = entry {
        let preview = render_content_preview(entry, theme, preview_viewport, preview_scrollable_id);

        let preview_section = container(preview)
            .width(Length::Fill)
            .height(Length::FillPortion(3))
            .padding(20);

        let date_str = if let Some(dt) = Local.timestamp_opt(entry.timestamp as i64, 0).latest() {
            dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "Unknown".to_string()
        };

        let mut metadata_column =
            column![text("Information").size(12).color(secondary_text_color),].spacing(8);

        if has_application_metadata {
            metadata_column = metadata_column.push(metadata_row(
                "Application",
                entry.window_title.clone().unwrap_or_default(),
                theme,
            ));
        }

        metadata_column = metadata_column
            .push(metadata_row(
                "Content type",
                match &entry.content {
                    ClipboardContent::Text(_) => "Text".to_string(),
                    ClipboardContent::Image(_) => "Image".to_string(),
                },
                theme,
            ))
            .push(metadata_row("Last copied", date_str, theme));

        let metadata_section = container(metadata_column)
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .padding(15)
            .style(|_| container::Style {
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });

        column![preview_section, metadata_section].into()
    } else {
        column![
            container(text("No item selected").color(secondary_text_color))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        ]
        .into()
    }
}

fn render_content_preview<'a>(
    entry: &'a ClipboardEntry,
    theme: &'a Theme,
    preview_viewport: Option<&'a iced::widget::scrollable::Viewport>,
    preview_scrollable_id: &'a iced::widget::Id,
) -> Element<'a, ClipboardHistoryMessage> {
    let text_color = theme.colors.text;

    match &entry.content {
        ClipboardContent::Text(t) => {
            let trimmed = t.trim();
            if trimmed.starts_with('#') && (trimmed.len() == 4 || trimmed.len() == 7) {
                let color = parse_hex_color(trimmed);
                let size = 76.0;
                let border_width = 4.0;

                container(
                    column![
                        container(
                            space()
                                .width(size + border_width * 2.0)
                                .height(size + border_width * 2.0)
                        )
                        .style(move |_| {
                            container::Style {
                                background: Some(color.into()),
                                border: iced::Border {
                                    radius: (size / 2.0 + border_width).into(),
                                    width: border_width,
                                    color: Color { a: 0.4, ..color },
                                },
                                ..Default::default()
                            }
                        }),
                        text(trimmed).size(15).color(text_color)
                    ]
                    .spacing(6)
                    .align_x(Alignment::Center),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                let lines: Vec<&str> = t.lines().collect();
                if lines.len() > 500 {
                    let row_height = LineHeight::default().to_absolute(14.0.into()).0;
                    let vp = preview_viewport;
                    let offset = vp.map(|v| v.absolute_offset().y).unwrap_or(0.0);
                    let height = vp.map(|v| v.bounds().height).unwrap_or(500.0);

                    let start_index = (offset / row_height).floor() as usize;
                    let visible_count = (height / row_height).ceil() as usize + 5;
                    let end_index = (start_index + visible_count).min(lines.len());

                    let top_spacer = space().height(Length::Fixed(start_index as f32 * row_height));
                    let bottom_spacer = space().height(Length::Fixed(
                        (lines.len().saturating_sub(end_index)) as f32 * row_height,
                    ));

                    let visible_lines: Vec<Element<_>> = lines
                        .iter()
                        .skip(start_index)
                        .take(end_index - start_index)
                        .map(|line| {
                            text(line.to_string())
                                .size(14)
                                .height(Length::Fixed(row_height))
                                .wrapping(iced::widget::text::Wrapping::None)
                                .color(text_color)
                                .into()
                        })
                        .collect();

                    let content = column![top_spacer]
                        .extend(visible_lines)
                        .push(bottom_spacer);

                    container(
                        scrollable(content)
                            .id(preview_scrollable_id.clone())
                            .on_scroll(ClipboardHistoryMessage::PreviewScrolled)
                            .width(Length::Fill),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
                } else {
                    container(
                        scrollable(text(t).size(14).color(text_color))
                            .id(preview_scrollable_id.clone())
                            .width(Length::Fill),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
                }
            }
        }
        ClipboardContent::Image(img) => {
            let handle = iced::widget::image::Handle::from_bytes(img.bytes.clone());

            container(image(handle).content_fit(iced::ContentFit::Contain))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    }
}

fn metadata_row<'a>(
    label: &'a str,
    value: String,
    theme: &'a Theme,
) -> Element<'a, ClipboardHistoryMessage> {
    row![
        text(label)
            .size(13)
            .color(theme.colors.text_60)
            .width(Length::Fill),
        text(value).size(13).color(theme.colors.text)
    ]
    .into()
}
