use iced::widget::{markdown, text};
use iced::{Element, Theme};

use crate::{components::types::DetailProps, message::Message};

pub fn render_detail<'a>(
    props: &'a DetailProps,
    parsed: &'a Vec<markdown::Item>,
) -> Element<'a, Message> {
    markdown::view(parsed, Theme::TokyoNight)
        .map(Message::LinkClicked)
        .into()
}
