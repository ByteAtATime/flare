use iced::{
    Border, Color, Element, Length, Theme, color,
    widget::{Button, button, column, container, mouse_area, opaque, row, text, text_input},
};

use crate::{
    components::actions::{Action, ActionPanelItem, ActionPanelSection},
    icons,
};

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");
const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

#[derive(Debug, Clone)]
pub enum ActionPanelMessage {
    Close,
    InvokeAction(crate::components::actions::ActionHandler),
    SearchChanged(String),
    Select(usize),
}

pub fn render_action_panel(
    actions: &[ActionPanelItem],
    search_text: &str,
    selected_index: usize,
) -> Element<'static, ActionPanelMessage> {
    let filtered_actions = filter_actions(actions, search_text);
    let search_text_owned = search_text.to_string();

    let mut current_index = 0usize;
    let actions_col = filtered_actions
        .into_iter()
        .fold(column![].spacing(4), |col, action| {
            let element = match action {
                ActionPanelItem::Action(action) => {
                    let is_selected = current_index == selected_index;
                    current_index += 1;
                    render_action_owned(action, is_selected, current_index - 1)
                }
                ActionPanelItem::Section(section) => {
                    render_section_owned(section, selected_index, &mut current_index)
                }
            };
            col.push(element)
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
                    background: Some(color!(0x2c2c2c).into()),
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

pub fn count_actions(actions: &[ActionPanelItem], search_text: &str) -> usize {
    let filtered = filter_actions(actions, search_text);
    filtered
        .iter()
        .map(|item| match item {
            ActionPanelItem::Action(_) => 1,
            ActionPanelItem::Section(section) => section.children.len(),
        })
        .sum()
}

pub fn filter_actions(actions: &[ActionPanelItem], search_text: &str) -> Vec<ActionPanelItem> {
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

fn render_action_owned(
    action: Action,
    is_selected: bool,
    index: usize,
) -> Element<'static, ActionPanelMessage> {
    let title = action.title.clone();
    let icon_char = action
        .icon
        .as_ref()
        .and_then(|icon_name| icons::get_icon(icon_name))
        .map(|s| s.to_string());

    let content: Element<'static, ActionPanelMessage> = if let Some(icon) = icon_char {
        row![
            text(icon).font(ICON_FONT).size(16),
            text(title).font(INTER_FONT).size(13)
        ]
        .spacing(8)
        .into()
    } else {
        text(title).font(INTER_FONT).size(13).into()
    };

    let mut btn = Button::new(content)
        .width(Length::Fill)
        .padding([6, 8])
        .style(move |_theme, _status| {
            if is_selected {
                button::Style {
                    background: Some(color!(0x007aff).into()),
                    text_color: Color::WHITE,
                    border: Border::default().rounded(4.0),
                    ..Default::default()
                }
            } else {
                button::Style {
                    background: None,
                    text_color: Color::WHITE,
                    border: Border::default().rounded(4.0),
                    ..Default::default()
                }
            }
        });

    if let Some(handler) = action.handler {
        btn = btn.on_press(ActionPanelMessage::InvokeAction(handler));
    } else {
        btn = btn.on_press(ActionPanelMessage::Select(index));
    }

    btn.into()
}

fn render_section_owned(
    section: ActionPanelSection,
    selected_index: usize,
    current_index: &mut usize,
) -> Element<'static, ActionPanelMessage> {
    let section_title = text(section.props.title.clone())
        .size(11)
        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.6))
        .font(INTER_FONT);

    let section_actions = section
        .children
        .into_iter()
        .fold(column![].spacing(2), |col, action| {
            let is_selected = *current_index == selected_index;
            let idx = *current_index;
            *current_index += 1;
            col.push(render_action_owned(action, is_selected, idx))
        });

    column![section_title, section_actions].spacing(4).into()
}

pub fn map_action_panel_message(msg: ActionPanelMessage) -> crate::Message {
    match msg {
        ActionPanelMessage::Close => crate::Message::ToggleActionPanel(false),
        ActionPanelMessage::InvokeAction(handler) => crate::Message::InvokeAction(handler),
        ActionPanelMessage::SearchChanged(text) => crate::Message::ActionPanelSearchChanged(text),
        ActionPanelMessage::Select(index) => crate::Message::ActionPanelSelect(index),
    }
}
