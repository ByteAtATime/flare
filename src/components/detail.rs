use iced::widget::markdown;
use iced::{Element, Theme};

use crate::components::types::DetailProps;
use crate::screens::detail::DetailMessage;

pub fn render_detail<'a>(
    props: &'a DetailProps,
    parsed: &'a Vec<markdown::Item>,
) -> Element<'a, DetailMessage> {
    markdown::view(parsed, Theme::TokyoNight)
        .map(DetailMessage::LinkClicked)
        .into()
}
