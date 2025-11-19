use iced::{
    Color, Element, Length, Theme,
    widget::{button, container, row, text},
};

use crate::{Message, state::State};

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

pub fn render_footer(state: &State) -> Element<'static, Message> {
    let primary_action = state.selected_actions.get(0);

    let action_button = if let Some(action) = primary_action {
        let callback_id = action.on_action.as_ref().map(|cb| cb.id.clone());

        let mut btn = button(text(action.title.clone()).font(INTER_FONT));

        if let Some(id) = callback_id {
            btn = btn.on_press(Message::InvokeAction(id));
        }

        Some(btn)
    } else {
        None
    };

    let action_panel_button = if state.selected_actions.len() >= 2 {
        Some(button(text("Actions").font(INTER_FONT)).on_press(Message::ToggleActionPanel(true)))
    } else {
        None
    };

    let mut footer_content =
        row![text(state.toast_message.clone()).width(Length::Fill)].spacing(10);

    if let Some(action_button) = action_button {
        footer_content = footer_content.push(action_button);
    }

    if let Some(action_panel_button) = action_panel_button {
        footer_content = footer_content.push(action_panel_button);
    }

    container(footer_content)
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
