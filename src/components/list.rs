use iced::widget::{column, container, row, space, text};
use iced::{Color, Element, Length, Theme};
use serde::Deserialize;

use super::actions::ActionPanel;
use super::dropdown::Dropdown;
use super::types::{CallbackInfo, deserialize_icon};
use crate::components::column as positionable_column;
use crate::components::detail::{DetailProps, render_detail};
use crate::icons;
use crate::position;
use crate::screens::list::ListMessage;

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");
const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListProps {
    #[serde(default)]
    pub props: ListProperties,
    #[serde(default, rename = "children")]
    pub sections: Vec<ListSectionProps>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListProperties {
    #[serde(default, rename = "onSearchTextChange")]
    pub on_search_text_change: Option<CallbackInfo>,
    #[serde(default, rename = "isShowingDetail")]
    pub is_showing_detail: bool,
    #[serde(default, rename = "searchBarAccessory")]
    pub search_bar_accessory: Option<Dropdown>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListSectionProps {
    #[serde(default)]
    pub props: ListSectionProperties,
    #[serde(default, rename = "children")]
    pub items: Vec<ListItemProps>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListSectionProperties {
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListItemProps {
    #[serde(default)]
    pub props: ListItemProperties,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListItemProperties {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default, deserialize_with = "deserialize_icon")]
    pub icon: Option<String>,
    #[serde(default)]
    pub actions: Option<ActionPanel>,
    #[serde(default)]
    pub detail: Option<DetailProps>,
}

pub fn render_list<'a>(
    props: &'a ListProps,
    selected_index: usize,
    column_id: position::Id,
    detail_cache: Option<&'a Vec<iced::widget::markdown::Item>>,
) -> Element<'a, ListMessage> {
    let mut item_cursor = 0;

    let list_view = props
        .sections
        .iter()
        .fold(
            positionable_column::Column::new().spacing(2).padding(10),
            |col, section| {
                let mut section_col = column![].spacing(2);

                if !section.props.title.is_empty() {
                    section_col =
                        section_col.push(text(section.props.title.clone()).font(INTER_FONT));
                }

                let start_idx = item_cursor;
                let items = section.items.iter().enumerate().map(|(idx, item)| {
                    let global_idx = start_idx + idx;
                    let is_selected = global_idx == selected_index;
                    render_list_item(item, is_selected)
                });

                item_cursor += section.items.len();

                col.push(section_col.extend(items))
            },
        )
        .id(column_id);

    if props.props.is_showing_detail {
        let mut detail_element: Element<'a, ListMessage> = container(space()).into();

        let mut current_idx = 0;
        'outer: for section in &props.sections {
            for item in &section.items {
                if current_idx == selected_index {
                    if let Some(detail) = &item.props.detail {
                        if let Some(items) = detail_cache {
                            detail_element = render_detail(detail, items).map(ListMessage::Detail);
                        }
                    }
                    break 'outer;
                }
                current_idx += 1;
            }
        }

        row![
            container(list_view).width(Length::FillPortion(1)),
            container(detail_element).width(Length::FillPortion(2))
        ]
        .into()
    } else {
        list_view.into()
    }
}

fn render_list_item(item: &ListItemProps, is_selected: bool) -> Element<'static, ListMessage> {
    let mut content = row![].spacing(12).align_y(iced::Alignment::Center);

    if let Some(icon_name) = &item.props.icon {
        if let Some(icon_char) = icons::get_icon(icon_name) {
            content = content.push(text(icon_char).font(ICON_FONT).width(20).center().style(
                move |_theme: &Theme| text::Style {
                    color: Some(if is_selected {
                        Color::WHITE
                    } else {
                        Color::from_rgb8(0xee, 0xee, 0xee)
                    }),
                    ..Default::default()
                },
            ));
        }
    }

    content = content.push(
        text(item.props.title.clone())
            .font(INTER_FONT)
            .width(Length::Fill),
    );

    if let Some(sub) = &item.props.subtitle {
        content = content.push(text(sub.clone()).font(INTER_FONT));
    }

    container(content)
        .width(Length::Fill)
        .padding([8, 12])
        .style(move |_theme: &Theme| container::Style {
            background: Some(if is_selected {
                Color::from_rgb8(0x00, 0x7A, 0xFF).into()
            } else {
                Color::TRANSPARENT.into()
            }),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
