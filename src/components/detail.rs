use iced::{
    Element, Theme,
    widget::{markdown, text},
};

use crate::{components::types::DetailProps, message::Message};

pub fn render_detail<'a>(props: &'a DetailProps) -> Element<'a, Message> {
    if let Some(parsed) = &props.parsed {
        markdown::view(
            parsed,
            markdown::Settings::default(),
            markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
        )
        .map(Message::LinkClicked)
        .into()
    } else {
        text(&props.markdown).into()
    }
}
