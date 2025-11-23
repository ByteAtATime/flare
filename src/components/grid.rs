use iced::widget::{column, container, image, row, scrollable, svg, text};
use iced::{Color, Element, Length, Theme};

use super::types::{GridItemContent, GridItemProps, GridProps, parse_hex_color};
use crate::components::column as positionable_column;
use crate::screens::grid::GridMessage;
use crate::{image_cache, position};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_grid(
    props: GridProps,
    selected_index: usize,
    column_id: position::Id,
    viewport: Option<&scrollable::Viewport>,
) -> Element<'static, GridMessage> {
    let layout_cache = crate::globals::LAYOUT_CACHE.lock().unwrap();

    let visible_range = viewport.map(|vp| {
        let offset_y = vp.absolute_offset().y;
        let height = vp.bounds().height;
        (offset_y - 1500.0, offset_y + height + 1500.0)
    });

    let mut item_cursor = 0;
    let mut col_child_idx = 0;

    let grid_view = props
        .sections
        .into_iter()
        .fold(
            positionable_column::Column::new().spacing(10).padding(10),
            |col, section| {
                let section_title = text(section.title.clone()).size(16).font(INTER_FONT);

                col_child_idx += 1;

                let columns = section.columns.or(props.columns).unwrap_or(5) as usize;

                let start_row_idx = col_child_idx;
                let row_count = (section.items.len() + columns - 1) / columns;
                col_child_idx += row_count;

                let rows: Vec<_> = section
                    .items
                    .chunks(columns)
                    .enumerate()
                    .map(|(chunk_idx, chunk)| {
                        let current_row_idx = start_row_idx + chunk_idx;

                        let is_visible = if let Some((start, end)) = visible_range {
                            if let Some(bounds) = layout_cache.get(&current_row_idx) {
                                let row_top = bounds.y;
                                let row_bottom = bounds.y + bounds.height;
                                row_bottom >= start && row_top <= end
                            } else {
                                current_row_idx < 10
                            }
                        } else {
                            current_row_idx < 10
                        };

                        let mut items_row = row![].spacing(10);
                        for (item_idx_in_row, item) in chunk.iter().enumerate() {
                            let global_idx = item_cursor + (chunk_idx * columns) + item_idx_in_row;
                            items_row = items_row.push(render_grid_item(
                                item.clone(),
                                global_idx == selected_index,
                                is_visible,
                            ));
                        }

                        if chunk.len() < columns {
                            items_row =
                                items_row
                                    .push(column![].width(Length::FillPortion(
                                        (columns - chunk.len()) as u16,
                                    )));
                        }
                        items_row.into()
                    })
                    .collect();

                item_cursor += section.items.len();

                col.push(section_title).extend(rows)
            },
        )
        .id(column_id);

    grid_view.into()
}

pub fn render_grid_item(
    props: GridItemProps,
    is_selected: bool,
    should_load_visibility: bool,
) -> Element<'static, GridMessage> {
    let border_color = if is_selected {
        Color::from_rgb8(0x00, 0x7A, 0xFF)
    } else {
        Color::from_rgb8(0x55, 0x55, 0x55)
    };

    let border_width = if is_selected { 3.0 } else { 2.0 };

    let content_widget: Element<'static, GridMessage> = match &props.content {
        Some(GridItemContent::Image(path)) => {
            if path.starts_with("http://") || path.starts_with("https://") {
                if let Some(handle) = image_cache::get(path) {
                    container(image(handle))
                        .center(150.0)
                        .style(move |_theme: &Theme| container::Style {
                            border: iced::Border {
                                color: border_color,
                                width: border_width,
                                radius: 8.0.into(),
                            },
                            ..Default::default()
                        })
                        .into()
                } else {
                    if should_load_visibility && image_cache::should_load(path) {
                        if let Some(loader) = crate::globals::IMAGE_LOADER.lock().unwrap().as_ref()
                        {
                            let _ = loader.send(path.clone());
                        }
                    }

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
            } else {
                container(if path.ends_with(".svg") {
                    Element::from(svg(path))
                } else {
                    Element::from(image(path))
                })
                .center(150.0)
                .style(move |_theme: &Theme| container::Style {
                    border: iced::Border {
                        color: border_color,
                        width: border_width,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .into()
            }
        }
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
