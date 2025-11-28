use iced::widget::{column, container, pick_list, row, rule, stack, text_input};
use iced::{Element, Length, Theme};

use crate::components::{
    action_panel::{render_action_panel, map_action_panel_message},
    dropdown::{Dropdown, DropdownChild},
    footer::{render_footer, map_footer_message},
};
use crate::message::Message;
use crate::screens::{Screen, Shell};
use crate::state::State;

#[derive(Debug, Clone, PartialEq)]
struct DropdownOption {
    title: String,
    value: String,
}

impl std::fmt::Display for DropdownOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let theme = &state.theme;
    let mut base_col = column![];

    if state.screen.can_search() {
        base_col = base_col.push(render_search_bar(state));
    }

    let content = match &state.screen {
        Screen::Root(s) => s.view(theme).map(Message::Root),
        Screen::Grid(s) => s.view().map(Message::Grid),
        Screen::Detail(s) => s.view().map(Message::Detail),
        Screen::List(s) => s.view().map(Message::List),
    };

    let footer = render_footer(&state.selected_actions, &state.toast_message, theme)
        .map(map_footer_message);

    base_col = base_col
        .push(container(content).width(Length::Fill).height(Length::Fill))
        .push(footer);

    let action_panel = if state.action_panel_visible {
        Some(render_action_panel(&state.selected_actions).map(map_action_panel_message))
    } else {
        None
    };

    let bg_color = theme.colors.background;

    container(stack![base_col, action_panel])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        })
        .into()
}

fn render_search_bar(state: &State) -> Element<'_, Message> {
    let theme = &state.theme;
    let text_color = theme.colors.text;

    let text_input = text_input("Search...", &state.search_text)
        .id(state.search_input_id.clone())
        .on_input(Message::SearchTextChanged)
        .size(16)
        .style(move |_theme: &Theme, status| {
            let base = text_input::default(_theme, status);
            text_input::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                value: text_color,
                placeholder: iced::Color {
                    a: 0.5,
                    ..text_color
                },
                selection: theme.colors.selection,
                ..base
            }
        });

    let mut row_content = row![text_input]
        .align_y(iced::Alignment::Center)
        .height(Length::Fill);

    if let Some(dropdown) = state.screen.get_search_bar_accessory() {
        row_content = row_content.push(render_dropdown_accessory(dropdown));
    }

    let search_bar = column![
        row_content.padding(10),
        rule::horizontal(1).style(|iced_theme: &iced::Theme| rule::Style {
            color: theme.colors.text_10,
            ..rule::default(iced_theme)
        })
    ]
    .width(Length::Fill)
    .height(60);

    container(search_bar).into()
}

fn render_dropdown_accessory(dropdown: &Dropdown) -> Element<'_, Message> {
    let options: Vec<DropdownOption> = dropdown
        .children
        .iter()
        .flat_map(|child| match child {
            DropdownChild::GridItem(item) | DropdownChild::ListItem(item) => {
                vec![DropdownOption {
                    title: item.props.title.clone(),
                    value: item.props.value.clone(),
                }]
            }
            DropdownChild::GridSection(section) | DropdownChild::ListSection(section) => section
                .children
                .iter()
                .map(|item| DropdownOption {
                    title: item.props.title.clone(),
                    value: item.props.value.clone(),
                })
                .collect(),
        })
        .collect();

    let selected_value = dropdown
        .props
        .value
        .as_ref()
        .or(dropdown.props.default_value.as_ref());

    let selected = selected_value
        .and_then(|val| options.iter().find(|opt| &opt.value == val).cloned())
        .or_else(|| options.first().cloned());

    pick_list(options, selected, |opt| Message::DropdownChanged(opt.value))
        .padding(10)
        .into()
}
