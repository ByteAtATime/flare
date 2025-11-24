use iced::widget::{button, column, container, markdown, row, scrollable, text};
use iced::{Border, Color, Element, Length, Theme};

use crate::components::types::{
    DetailMetadata, DetailMetadataItem, DetailProps, MetadataTagListItem, parse_hex_color,
};
use crate::icons;
use crate::screens::detail::DetailMessage;

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

fn render_metadata<'a>(metadata: &'a DetailMetadata) -> iced::Element<'a, DetailMessage> {
    println!("rendering");
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
    let metadata = if let Some(metadata) = &props.metadata {
        Some(render_metadata(&metadata))
    } else {
        None
    };

    row![
        scrollable(
            container(markdown::view(parsed, Theme::TokyoNight).map(DetailMessage::LinkClicked))
                .padding(20)
                .width(Length::Fill),
        )
        .height(Length::Fill),
        metadata
    ]
    .into()
}
