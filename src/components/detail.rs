use iced::widget::image;
use iced::widget::{button, column, container, markdown, row, scrollable, space, text};
use iced::{Border, Color, Element, Length, Theme};
use serde::Deserialize;

use super::actions::ActionPanel;
use super::types::{CallbackInfo, deserialize_icon};
use crate::icons;
use crate::image_cache::get;
use crate::screens::detail::DetailMessage;

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DetailProps {
    #[serde(default)]
    pub props: DetailProperties,
    #[serde(skip)]
    pub parsed: Option<Vec<markdown::Item>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DetailProperties {
    #[serde(default)]
    pub markdown: String,
    #[serde(default)]
    pub metadata: Option<DetailMetadata>,
    #[serde(default)]
    pub actions: Option<ActionPanel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailMetadata {
    #[serde(rename = "children")]
    pub items: Vec<DetailMetadataItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DetailMetadataItem {
    #[serde(rename = "Detail.Metadata.Label")]
    Label { props: MetadataLabelProps },

    #[serde(rename = "Detail.Metadata.Link")]
    Link { props: MetadataLinkProps },

    #[serde(rename = "Detail.Metadata.TagList")]
    TagList {
        props: MetadataTagListProps,
        children: Vec<MetadataTagListItem>,
    },

    #[serde(rename = "Detail.Metadata.Separator")]
    Separator,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetadataLabelProps {
    pub title: String,
    pub text: Option<String>,
    #[serde(default, deserialize_with = "deserialize_icon")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetadataLinkProps {
    pub title: String,
    pub text: String,
    pub target: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetadataTagListProps {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum MetadataTagListItem {
    #[serde(rename = "Detail.Metadata.TagList.Item")]
    Item { props: MetadataTagListItemProps },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetadataTagListItemProps {
    pub text: Option<String>,
    pub color: Option<String>,
    #[serde(default, deserialize_with = "deserialize_icon")]
    pub icon: Option<String>,
    #[serde(rename = "onAction")]
    pub on_action: Option<CallbackInfo>,
}

fn render_metadata<'a>(metadata: &'a DetailMetadata) -> iced::Element<'a, DetailMessage> {
    let metadata_items = metadata
        .items
        .iter()
        .fold(column![].spacing(5), |col, item| match item {
            DetailMetadataItem::Label { props } => col.push(row![
                text(format!("{}: ", props.title)),
                text(props.text.clone().unwrap_or_default())
            ]),
            DetailMetadataItem::Link { props } => col.push(column![
                text(format!("{}: ", props.title)),
                button(text(props.text.clone()))
                    .on_press(DetailMessage::LinkClicked(props.target.clone()))
                    .style(|_theme, _status| button::Style {
                        background: None,
                        border: Border::default(),
                        text_color: Color::from_rgb8(255, 255, 255).into(),
                        ..Default::default()
                    })
                    .padding(0)
            ]),
            DetailMetadataItem::TagList { props, children } => {
                let tags = children
                    .iter()
                    .fold(row![].spacing(5), |row, item| match item {
                        MetadataTagListItem::Item { props } => {
                            let mut tag_content =
                                row![].spacing(4).align_y(iced::Alignment::Center);

                            if let Some(icon_name) = &props.icon {
                                if let Some(icon_char) = icons::get_icon(icon_name) {
                                    tag_content = tag_content.push(text(icon_char).font(ICON_FONT));
                                }
                            }

                            if let Some(text_content) = &props.text {
                                tag_content = tag_content.push(text(text_content));
                            }

                            row.push(container(tag_content).padding([2, 6]).style(
                                move |_theme: &Theme| container::Style {
                                    border: Border {
                                        radius: 4.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                            ))
                        }
                    });

                col.push(
                    row![text(format!("{}: ", props.title)), tags].align_y(iced::Alignment::Center),
                )
            }
            DetailMetadataItem::Separator => col.push(
                container(row![])
                    .height(1)
                    .width(Length::Fill)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(Color::from_rgba8(255, 255, 255, 0.1).into()),
                        ..Default::default()
                    }),
            ),
        });

    container(metadata_items)
        .padding(10)
        .width(256)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::from_rgb8(0x18, 0x18, 0x18).into()),
            ..Default::default()
        })
        .into()
}

pub fn render_detail<'a>(
    props: &'a DetailProps,
    parsed: &'a Vec<markdown::Item>,
) -> Element<'a, DetailMessage> {
    let metadata = if let Some(metadata) = &props.props.metadata {
        Some(render_metadata(&metadata))
    } else {
        None
    };

    row![
        scrollable(
            container(markdown::view_with(
                parsed,
                Theme::TokyoNight,
                &MarkdownViewer {}
            ))
            .padding(20)
            .width(Length::Fill),
        )
        .height(Length::Fill),
        metadata
    ]
    .into()
}

struct MarkdownViewer {}

impl<'a> markdown::Viewer<'a, DetailMessage> for MarkdownViewer {
    fn on_link_click(url: markdown::Uri) -> DetailMessage {
        DetailMessage::LinkClicked(url)
    }

    fn image(
        &self,
        _settings: markdown::Settings,
        url: &'a markdown::Uri,
        _title: &'a str,
        _alt: &markdown::Text,
    ) -> Element<'a, DetailMessage> {
        if let Some(handle) = get(url) {
            container(image(handle)).center(150.0).into()
        } else {
            container(space()).into()
        }
    }
}
