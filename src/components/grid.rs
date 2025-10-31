use iced::widget::{column, container, row, text};
use iced::{Color, Element, Theme};

use super::types::{GridItemProps, GridProps, GridSectionProps, parse_hex_color};
use crate::Message;

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_grid(props: GridProps, selected_index: usize) -> Element<'static, Message> {
    props
        .sections
        .into_iter()
        .fold(column![], |col, section| {
            col.push(render_grid_section(section, selected_index))
        })
        .into()
}

pub fn render_grid_section(
    props: GridSectionProps,
    selected_index: usize,
) -> Element<'static, Message> {
    let items_row = props
        .items
        .into_iter()
        .enumerate()
        .fold(row![].spacing(10), |row, (idx, item)| {
            row.push(render_grid_item(item, idx == selected_index))
        });

    column![text(props.title).size(16).font(INTER_FONT), items_row]
        .padding(10)
        .spacing(10)
        .into()
}

pub fn render_grid_item(props: GridItemProps, is_selected: bool) -> Element<'static, Message> {
    let bg_color = props
        .content
        .as_ref()
        .and_then(|c| c.color.as_ref())
        .map(|color| parse_hex_color(&color.light))
        .unwrap_or(Color::from_rgb8(0x33, 0x33, 0x33));

    let border_color = if is_selected {
        Color::from_rgb8(0x00, 0x7A, 0xFF)
    } else {
        Color::from_rgb8(0x55, 0x55, 0x55)
    };

    let border_width = if is_selected { 3.0 } else { 2.0 };

    container(
        column![
            container(text(""))
                .width(150)
                .height(150)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: border_color,
                        width: border_width,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            text(props.title).size(14).font(INTER_FONT),
            text(props.subtitle.unwrap_or_default())
                .size(12)
                .font(INTER_FONT)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb8(0xaa, 0xaa, 0xaa)),
                    ..Default::default()
                })
        ]
        .spacing(5),
    )
    .into()
}
