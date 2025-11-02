use iced::widget::{column, container, image, row, text};
use iced::{Color, Element, Length, Theme};

use super::types::{GridItemContent, GridItemProps, GridProps, parse_hex_color};
use crate::components::column as positionable_column;
use crate::{Message, position};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_grid(
    props: GridProps,
    selected_index: usize,
    column_id: position::Id,
) -> Element<'static, Message> {
    let mut item_cursor = 0;
    let grid_view = props
        .sections
        .into_iter()
        .fold(
            positionable_column::Column::new().spacing(10).padding(10),
            |col, section| {
                let section_title = text(section.title.clone()).size(16).font(INTER_FONT);

                let columns = section.columns.or(props.columns).unwrap_or(5) as usize;

                let items_grid = section.items.chunks(columns).enumerate().fold(
                    column![].spacing(10),
                    |rows_col, (chunk_idx, chunk)| {
                        let mut items_row = row![].spacing(10);
                        for (item_idx_in_row, item) in chunk.iter().enumerate() {
                            let global_idx = item_cursor + (chunk_idx * columns) + item_idx_in_row;
                            items_row = items_row
                                .push(render_grid_item(item.clone(), global_idx == selected_index));
                        }
                        if chunk.len() < columns {
                            items_row =
                                items_row
                                    .push(column![].width(Length::FillPortion(
                                        (columns - chunk.len()) as u16,
                                    )));
                        }
                        rows_col.push(items_row)
                    },
                );

                item_cursor += section.items.len();
                col.push(section_title).push(items_grid)
            },
        )
        .id(column_id);

    grid_view.into()
}

pub fn render_grid_item(props: GridItemProps, is_selected: bool) -> Element<'static, Message> {
    let border_color = if is_selected {
        Color::from_rgb8(0x00, 0x7A, 0xFF)
    } else {
        Color::from_rgb8(0x55, 0x55, 0x55)
    };

    let border_width = if is_selected { 3.0 } else { 2.0 };

    let content_widget: Element<'static, Message> = match &props.content {
        Some(GridItemContent::Image(path)) => container(image(path))
            .center(150.0)
            .style(move |_theme: &Theme| container::Style {
                border: iced::Border {
                    color: border_color,
                    width: border_width,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .into(),
        Some(GridItemContent::Color(color)) => {
            let bg_color = parse_hex_color(&color.light);
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
                })
                .into()
        }
        None => {
            let bg_color = Color::from_rgb8(0x33, 0x33, 0x33);
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
                })
                .into()
        }
    };

    container(
        column![
            content_widget,
            text(props.title).size(14).font(INTER_FONT),
            text(props.subtitle.unwrap_or_default())
                .size(12)
                .font(INTER_FONT)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb8(0xaa, 0xaa, 0xaa)),
                    ..Default::default()
                })
        ]
        .width(Length::FillPortion(1))
        .spacing(5),
    )
    .into()
}
