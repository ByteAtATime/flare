use iced::widget::{button, column, container, markdown, row, scrollable, text};
use iced::{Border, Color, Element, Length, Theme};

use crate::components::types::{DetailMetadata, DetailMetadataItem, DetailProps};
use crate::screens::detail::DetailMessage;

fn render_metadata<'a>(metadata: &DetailMetadata) -> iced::Element<'a, DetailMessage> {
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
