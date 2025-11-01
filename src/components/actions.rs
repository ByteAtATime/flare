use iced::{
    Color, Length,
    widget::{Button, column, container, mouse_area, opaque, text},
};

use crate::Message;

pub fn render_action_panel(state: &crate::State) -> iced::Element<'_, crate::Message> {
    let actions = state
        .selected_actions
        .iter()
        .fold(column![].spacing(10), |col, action| {
            col.push({
                let mut button = Button::new(text(action.title.clone()));

                if let Some(callback) = &action.on_action {
                    button = button.on_press(Message::InvokeAction(callback.id.clone()));
                }

                button
            })
        });

    opaque(mouse_area(
        container(
            column![
                container(opaque(actions))
                    .padding(8)
                    .style(|_theme| container::Style {
                        background: Some(Color::from_rgba(0.1, 0.1, 0.1, 0.95).into()),
                        ..Default::default()
                    }),
                container(column![]).height(40)
            ]
            .spacing(10),
        )
        .align_bottom(Length::Fill)
        .align_right(Length::Fill),
    ))
    .into()
}
