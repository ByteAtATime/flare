use iced::{
    Color, Element, Length,
    widget::{Button, column, container, mouse_area, opaque, row, text},
};

use crate::{
    Message,
    components::types::{Action, ActionPanelItem},
    icons,
};

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

fn render_action(action: &Action) -> iced::Element<'_, crate::Message> {
    let mut button = Button::new(
        if let Some(icon) = action
            .icon
            .as_ref()
            .and_then(|icon_name| icons::get_icon(icon_name))
        {
            row![text(icon).font(ICON_FONT), text(action.title.clone())].into()
        } else {
            Element::from(text(action.title.clone()))
        },
    );

    if let Some(callback) = &action.on_action {
        button = button.on_press(Message::InvokeAction(callback.id.clone()));
    }

    button.into()
}

pub fn render_action_panel(state: &crate::State) -> iced::Element<'_, crate::Message> {
    let actions = state
        .selected_actions
        .iter()
        .fold(column![].spacing(10), |col, action| {
            col.push({
                match action {
                    ActionPanelItem::Action(action) => render_action(action),
                    ActionPanelItem::Section(section) => {
                        let section_title = text(&section.title)
                            .size(16)
                            .color(Color::from_rgb8(0xFF, 0xFF, 0xFF));

                        let section_actions = section
                            .children
                            .iter()
                            .fold(column![].spacing(5), |col, action| {
                                col.push(render_action(action))
                            });

                        column![section_title, section_actions].spacing(5).into()
                    }
                }
            })
        });

    opaque(
        mouse_area(
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
        )
        .on_press(Message::ToggleActionPanel(false)),
    )
    .into()
}
