use iced::{
    Color, Element, Length, Theme,
    widget::{button, container, row, text},
};

use crate::{Message, State};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_footer(state: &State) -> Element<'static, Message> {
    let first_action = state.selected_actions.get(0);

    let action_button = if let Some(action) = first_action {
        let callback_id = action.on_action.as_ref().map(|cb| cb.id.clone());

        let mut btn = button(text(action.title.clone()).font(INTER_FONT));

        if let Some(id) = callback_id {
            btn = btn.on_press(Message::InvokeAction(id));
        }

        btn
    } else {
        button(text("No Action").font(INTER_FONT))
    };

    container(row![
        text(state.toast_message.clone()).width(Length::Fill),
        action_button
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
