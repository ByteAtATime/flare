use iced::widget::{column, container, pick_list, row, stack, text_input};
use iced::{Element, Length, Theme};

use crate::components::{
    actions::render_action_panel,
    dropdown::{Dropdown, DropdownChild},
    footer::render_footer,
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

    base_col = base_col
        .push(container(content).width(Length::Fill).height(Length::Fill))
        .push(render_footer(state, theme));

    let action_panel = if state.action_panel_visible {
        Some(render_action_panel(state))
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
        .size(20)
        .padding(12)
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

    let mut row_content = row![text_input].align_y(iced::Alignment::Center);

    if let Some(dropdown) = state.screen.get_search_bar_accessory() {
        row_content = row_content.push(render_dropdown_accessory(dropdown));
    }

    let border_color = iced::Color {
        a: 0.2,
        ..text_color
    };

    container(row_content)
        .padding(10)
        .style(move |_theme: &Theme| container::Style {
            border: iced::Border {
                color: border_color,
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
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
