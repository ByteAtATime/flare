use iced::{
    Color, Element, Length, Theme,
    widget::{button, container, row, text},
};

use crate::{Message, State};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_footer(state: &State) -> Element<'static, Message> {
    container(row![
        text(state.toast_message.clone()).width(Length::Fill),
        button(
            text(
                state
                    .selected_actions
                    .get(0)
                    .map_or("No Action".to_string(), |action| { action.title.clone() })
            )
            .font(INTER_FONT)
        )
    ])
    .width(Length::Fill)
    .padding([0, 8])
    .center_y(40)
    .style(|_theme: &Theme| container::Style {
        background: Some(Color::from_rgb8(0x22, 0x22, 0x22).into()),
        text_color: Some(Color::WHITE),
        ..Default::default()
    })
    .into()
}
