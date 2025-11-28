use iced::{
    Color, Element, Length,
    widget::{Button, column, container, mouse_area, opaque, row, text},
};

use crate::{
    components::actions::{Action, ActionPanelItem, ActionPanelSection},
    icons,
};

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

#[derive(Debug, Clone)]
pub enum ActionPanelMessage {
    Close,
    InvokeAction(crate::components::actions::ActionHandler),
}

pub fn render_action_panel(actions: &[ActionPanelItem]) -> Element<'_, ActionPanelMessage> {
    let actions_col = actions
        .iter()
        .fold(column![].spacing(10), |col, action| {
            col.push(match action {
                ActionPanelItem::Action(action) => render_action(action),
                ActionPanelItem::Section(section) => render_section(section),
            })
        });

    opaque(
        mouse_area(
            container(
                column![
                    container(opaque(actions_col))
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
        .on_press(ActionPanelMessage::Close),
    )
    .into()
}

fn render_action(action: &Action) -> Element<'_, ActionPanelMessage> {
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

    if let Some(handler) = &action.handler {
        button = button.on_press(ActionPanelMessage::InvokeAction(handler.clone()));
    }

    button.into()
}

fn render_section(section: &ActionPanelSection) -> Element<'_, ActionPanelMessage> {
    let section_title = text(&section.props.title)
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

pub fn map_action_panel_message(msg: ActionPanelMessage) -> crate::Message {
    match msg {
        ActionPanelMessage::Close => crate::Message::ToggleActionPanel(false),
        ActionPanelMessage::InvokeAction(handler) => crate::Message::InvokeAction(handler),
    }
}
