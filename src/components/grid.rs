use iced::widget::{column, container, image, row, text};
use iced::{Color, Element, Length, Theme};

use super::types::{GridItemContent, GridItemProps, GridProps, GridSectionProps, parse_hex_color};
use crate::Message;

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_grid(props: GridProps, selected_index: usize) -> Element<'static, Message> {
    let mut global_index = 0;
    let grid_columns = props.columns;
    props
        .sections
        .into_iter()
        .fold(column![], |col, section| {
            let section_len = section.items.len();
            let section_element =
                render_grid_section(section, selected_index, global_index, grid_columns);
            global_index += section_len;
            col.push(section_element)
        })
        .into()
}

fn render_grid_section(
    props: GridSectionProps,
    selected_index: usize,
    base_index: usize,
    grid_columns: Option<i32>,
) -> Element<'static, Message> {
    let columns = props.columns.or(grid_columns).unwrap_or(5) as usize;

    let mut rows_column = column![].spacing(10);

    for (chunk_idx, chunk) in props.items.chunks(columns).enumerate() {
        let mut items_row = row![].spacing(10);
        for (idx, item) in chunk.iter().enumerate() {
            let global_idx = base_index + (chunk_idx * columns) + idx;
            items_row =
                items_row.push(render_grid_item(item.clone(), global_idx == selected_index));
        }
        if chunk.len() < columns {
            items_row = items_row
                .push(column![].width(Length::FillPortion((columns - chunk.len()) as u16)));
        }
        rows_column = rows_column.push(items_row);
    }

    column![text(props.title).size(16).font(INTER_FONT), rows_column]
        .padding(10)
        .spacing(10)
        .into()
}

pub fn render_grid_item(props: GridItemProps, is_selected: bool) -> Element<'static, Message> {
    let border_color = if is_selected {
        Color::from_rgb8(0x00, 0x7A, 0xFF)
    } else {
        Color::from_rgb8(0x55, 0x55, 0x55)
    };

    let border_width = if is_selected { 3.0 } else { 2.0 };

    let content_widget: Element<'static, Message> = match &props.content {
        Some(GridItemContent::Image(path)) => container(image(path).width(150).height(150))
            .width(150)
            .height(150)
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
