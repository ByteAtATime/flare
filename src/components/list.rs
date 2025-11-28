use iced::widget::{container, row, scrollable, space, text};
use iced::{Color, Element, Length, Theme};
use serde::Deserialize;
use serde_json::Value;

use super::actions::ActionPanel;
use super::dropdown::Dropdown;
use super::types::{CallbackInfo, deserialize_icon, parse_hex_color};
use crate::components::column as positionable_column;
use crate::components::detail::{DetailProps, render_detail};
use crate::icons;
use crate::position;
use crate::screens::list::ListMessage;

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");
const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EmptyViewProps {
    #[serde(default)]
    pub props: EmptyViewProperties,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EmptyViewProperties {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ListChild {
    #[serde(rename = "List.Section")]
    Section(ListSectionProps),
    #[serde(rename = "List.Item")]
    Item(ListItemProps),
    #[serde(rename = "List.EmptyView")]
    EmptyView(EmptyViewProps),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(from = "ListPropsRaw")]
pub struct ListProps {
    pub props: ListProperties,
    pub sections: Vec<ListSectionProps>,
    pub empty_view: Option<EmptyViewProps>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ListPropsRaw {
    #[serde(default)]
    props: ListProperties,
    #[serde(default)]
    children: Vec<ListChild>,
}

impl From<ListPropsRaw> for ListProps {
    fn from(raw: ListPropsRaw) -> Self {
        let mut sections = Vec::new();
        let mut empty_view = None;
        let mut loose_items = Vec::new();

        for child in raw.children {
            match child {
                ListChild::Section(section) => sections.push(section),
                ListChild::Item(item) => loose_items.push(item),
                ListChild::EmptyView(ev) => empty_view = Some(ev),
                ListChild::Unknown => {}
            }
        }

        if !loose_items.is_empty() {
            sections.insert(
                0,
                ListSectionProps {
                    props: ListSectionProperties::default(),
                    items: loose_items,
                },
            );
        }

        ListProps {
            props: raw.props,
            sections,
            empty_view,
        }
    }
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
    #[serde(default)]
    pub accessories: Vec<ListItemAccessory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListItemAccessory {
    #[serde(default, deserialize_with = "deserialize_accessory_value")]
    pub text: Option<AccessoryValue>,
    #[serde(default, deserialize_with = "deserialize_accessory_value")]
    pub date: Option<AccessoryValue>,
    #[serde(default, deserialize_with = "deserialize_accessory_value")]
    pub tag: Option<AccessoryValue>,
    #[serde(default, deserialize_with = "deserialize_icon")]
    pub icon: Option<String>,
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AccessoryValue {
    pub value: String,
    pub color: Option<Color>,
}

fn parse_color(s: &str) -> Color {
    if s.starts_with("#") {
        return parse_hex_color(s);
    }
    match s {
        "red" => Color::from_rgb8(255, 59, 48),
        "orange" => Color::from_rgb8(255, 149, 0),
        "yellow" => Color::from_rgb8(255, 204, 0),
        "green" => Color::from_rgb8(40, 205, 65),
        "blue" => Color::from_rgb8(0, 122, 255),
        "purple" => Color::from_rgb8(175, 82, 222),
        "magenta" => Color::from_rgb8(175, 82, 222),
        _ => Color::from_rgb(0.5, 0.5, 0.5),
    }
}

pub fn deserialize_accessory_value<'de, D>(
    deserializer: D,
) -> Result<Option<AccessoryValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Value = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(Some(AccessoryValue {
            value: s,
            color: None,
        })),
        Value::Object(map) => {
            let value = if let Some(val) = map.get("value") {
                val.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| Some(val.to_string()))
                    .unwrap()
            } else if let Some(text) = map.get("text") {
                text.as_str().map(|s| s.to_string()).unwrap_or_default()
            } else {
                return Ok(None);
            };

            let color = if let Some(c) = map.get("color") {
                if let Some(s) = c.as_str() {
                    Some(parse_color(s))
                } else if let Some(obj) = c.as_object() {
                    if let Some(l) = obj.get("light").and_then(|v| v.as_str()) {
                        Some(parse_color(l))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            Ok(Some(AccessoryValue { value, color }))
        }
        _ => Ok(None),
    }
}

fn render_empty_view(empty_view: &EmptyViewProps) -> Element<'static, ListMessage> {
    let mut col = iced::widget::column![]
        .spacing(8)
        .align_x(iced::Alignment::Center);

    if let Some(title) = &empty_view.props.title {
        col = col.push(
            text(title.clone())
                .font(INTER_FONT)
                .size(16)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::WHITE),
                    ..Default::default()
                }),
        );
    }

    if let Some(description) = &empty_view.props.description {
        col = col.push(text(description.clone()).font(INTER_FONT).size(13).style(
            |_theme: &Theme| text::Style {
                color: Some(Color::from_rgb8(0x88, 0x88, 0x88)),
                ..Default::default()
            },
        ));
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn render_list<'a>(
    props: &'a ListProps,
    selected_index: usize,
    column_id: position::Id,
    scrollable_id: iced::widget::Id,
    detail_cache: Option<&'a Vec<iced::widget::markdown::Item>>,
) -> Element<'a, ListMessage> {
    let total_items: usize = props.sections.iter().map(|s| s.items.len()).sum();

    if total_items == 0 {
        if let Some(empty_view) = &props.empty_view {
            return render_empty_view(empty_view);
        }
    }

    let mut item_cursor = 0;

    let mut col = positionable_column::Column::new().spacing(2).padding(10);

    for section in &props.sections {
        if !section.props.title.is_empty() {
            col = col.push(text(section.props.title.clone()).font(INTER_FONT));
        }

        for (idx, item) in section.items.iter().enumerate() {
            let global_idx = item_cursor + idx;
            let is_selected = global_idx == selected_index;
            col = col.push(render_list_item(item, is_selected));
        }
        item_cursor += section.items.len();
    }

    let list_view = col.id(column_id);

    let scrollable_list = scrollable(list_view)
        .id(scrollable_id)
        .on_scroll(ListMessage::Scrolled)
        .height(Length::Fill);

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
            container(scrollable_list)
                .width(Length::FillPortion(1))
                .height(Length::Fill),
            container(detail_element)
                .width(Length::FillPortion(2))
                .height(Length::Fill)
        ]
        .height(Length::Fill)
        .into()
    } else {
        scrollable_list.into()
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

    if !item.props.accessories.is_empty() {
        let mut accessories_row = row![].spacing(8).align_y(iced::Alignment::Center);

        for accessory in &item.props.accessories {
            let mut acc_content = row![].spacing(4).align_y(iced::Alignment::Center);

            if let Some(icon_name) = &accessory.icon {
                if let Some(icon_char) = icons::get_icon(icon_name) {
                    acc_content = acc_content.push(text(icon_char).font(ICON_FONT).size(12).style(
                        move |_theme: &Theme| text::Style {
                            color: Some(if is_selected {
                                Color::WHITE
                            } else {
                                Color::from_rgb8(0x88, 0x88, 0x88)
                            }),
                            ..Default::default()
                        },
                    ));
                }
            }

            if let Some(txt) = &accessory.text {
                let color = if is_selected {
                    Some(Color::WHITE)
                } else {
                    txt.color.or(Some(Color::from_rgb8(0x88, 0x88, 0x88)))
                };

                acc_content =
                    acc_content.push(text(txt.value.clone()).font(INTER_FONT).size(12).style(
                        move |_| text::Style {
                            color,
                            ..Default::default()
                        },
                    ));
            }

            if let Some(date) = &accessory.date {
                let color = if is_selected {
                    Some(Color::WHITE)
                } else {
                    date.color.or(Some(Color::from_rgb8(0x88, 0x88, 0x88)))
                };
                acc_content =
                    acc_content.push(text(date.value.clone()).font(INTER_FONT).size(12).style(
                        move |_| text::Style {
                            color,
                            ..Default::default()
                        },
                    ));
            }

            if let Some(tag) = &accessory.tag {
                let tag_color = if is_selected {
                    Color::WHITE
                } else {
                    tag.color.unwrap_or(Color::from_rgb8(0x88, 0x88, 0x88))
                };
                let bg = if is_selected {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                } else {
                    Color {
                        a: 0.1,
                        ..tag_color
                    }
                };

                acc_content = acc_content.push(
                    container(
                        text(tag.value.clone())
                            .size(10)
                            .style(move |_| text::Style {
                                color: Some(tag_color),
                                ..Default::default()
                            }),
                    )
                    .padding([2, 6])
                    .style(move |_| container::Style {
                        background: Some(bg.into()),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                );
            }

            accessories_row = accessories_row.push(acc_content);
        }

        content = content.push(accessories_row);
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
