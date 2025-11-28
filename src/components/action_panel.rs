use iced::{
    Border, Color, Element, Length, Theme, color,
    widget::{Button, column, container, mouse_area, opaque, row, text, text_input},
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
    SearchChanged(String),
}

pub fn render_action_panel(
    actions: &[ActionPanelItem],
    search_text: &str,
) -> Element<'static, ActionPanelMessage> {
    let filtered_actions = filter_actions(actions, search_text);
    let search_text_owned = search_text.to_string();

    let actions_col = filtered_actions
        .into_iter()
        .fold(column![].spacing(10), |col, action| {
            col.push(match action {
                ActionPanelItem::Action(action) => render_action_owned(action),
                ActionPanelItem::Section(section) => render_section_owned(section),
            })
        });

    let search_bar = text_input("Search for actions...", &search_text_owned)
        .on_input(ActionPanelMessage::SearchChanged)
        .size(14)
        .padding(8)
        .style(|_theme: &Theme, _status| text_input::Style {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: iced::Border::default(),
            icon: Color::from_rgba(1.0, 1.0, 1.0, 0.6),
            placeholder: Color::from_rgba(1.0, 1.0, 1.0, 0.4),
            value: Color::WHITE,
            selection: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
        });

    let action_panel = column![
        container(opaque(column![actions_col, search_bar,]))
            .padding(8)
            .style(|_theme| {
                container::Style {
                    background: Some(color!(0x2c2c2c).into()), // TODO: where does this color come from?
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                }
            }),
        container(column![]).height(40)
    ]
    .width(Length::Fixed(368.0))
    .spacing(10);

    opaque(
        mouse_area(
            container(action_panel)
                .align_bottom(Length::Fill)
                .align_right(Length::Fill),
        )
        .on_press(ActionPanelMessage::Close),
    )
    .into()
}

fn filter_actions(actions: &[ActionPanelItem], search_text: &str) -> Vec<ActionPanelItem> {
    if search_text.is_empty() {
        return actions.to_vec();
    }

    let search_lower = search_text.to_lowercase();

    actions
        .iter()
        .filter_map(|item| match item {
            ActionPanelItem::Action(action) => {
                if action.title.to_lowercase().contains(&search_lower) {
                    Some(ActionPanelItem::Action(action.clone()))
                } else {
                    None
                }
            }
            ActionPanelItem::Section(section) => {
                let filtered_children: Vec<Action> = section
                    .children
                    .iter()
                    .filter(|a| a.title.to_lowercase().contains(&search_lower))
                    .cloned()
                    .collect();

                if filtered_children.is_empty() {
                    None
                } else {
                    Some(ActionPanelItem::Section(ActionPanelSection {
                        props: section.props.clone(),
                        children: filtered_children,
                    }))
                }
            }
        })
        .collect()
}

fn render_action_owned(action: Action) -> Element<'static, ActionPanelMessage> {
    let title = action.title.clone();
    let icon_char = action
        .icon
        .as_ref()
        .and_then(|icon_name| icons::get_icon(icon_name))
        .map(|s| s.to_string());

    let mut button = Button::new(if let Some(icon) = icon_char {
        row![text(icon).font(ICON_FONT), text(title)].into()
    } else {
        Element::from(text(title))
    });

    if let Some(handler) = action.handler {
        button = button.on_press(ActionPanelMessage::InvokeAction(handler));
    }

    button.into()
}

fn render_section_owned(section: ActionPanelSection) -> Element<'static, ActionPanelMessage> {
    let section_title = text(section.props.title.clone())
        .size(16)
        .color(Color::from_rgb8(0xFF, 0xFF, 0xFF));

    let section_actions = section
        .children
        .into_iter()
        .fold(column![].spacing(5), |col, action| {
            col.push(render_action_owned(action))
        });

    column![section_title, section_actions].spacing(5).into()
}

pub fn map_action_panel_message(msg: ActionPanelMessage) -> crate::Message {
    match msg {
        ActionPanelMessage::Close => crate::Message::ToggleActionPanel(false),
        ActionPanelMessage::InvokeAction(handler) => crate::Message::InvokeAction(handler),
        ActionPanelMessage::SearchChanged(text) => crate::Message::ActionPanelSearchChanged(text),
    }
}
