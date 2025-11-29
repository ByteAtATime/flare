use iced::widget::text;
use iced::{
    Alignment::Center,
    Border, Color, Element, Length, Padding, Task, Theme, color,
    widget::{
        Button, button, column, container, mouse_area, opaque, row, rule, space, text::LineHeight,
        text_input,
    },
};
use std::time::Instant;

use crate::{
    components::actions::{Action, ActionHandler, ActionPanelItem, ActionPanelSection},
    icons,
};

pub mod animation {
    pub const DURATION_MS: u64 = 100; // below this feels choppy, above is sluggish
    pub const OPACITY_START: f32 = 0.0;
    pub const SCALE_START: f32 = 0.95;
}

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");
const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

const SELECTED_BG: Color = Color::from_rgb(0.0, 0.48, 1.0);
const SECTION_TITLE_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.5);
const TEXT_COLOR: Color = Color::WHITE;

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    InvokeAction(ActionHandler),
    SearchChanged(String),
    Select(usize),
    MoveUp,
    MoveDown,
    InvokeSelected,
    Tick(Instant),
}

#[derive(Debug, Clone, Default)]
pub struct AnimationState {
    pub start_time: Option<Instant>,
    pub opacity: f32,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct State {
    pub visible: bool,
    pub search_text: String,
    pub selected_index: usize,
    pub input_id: iced::widget::Id,
    pub animation: AnimationState,
}

impl State {
    pub fn new() -> Self {
        Self {
            visible: false,
            search_text: String::new(),
            selected_index: 0,
            input_id: iced::widget::Id::unique(),
            animation: AnimationState::default(),
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        actions: &[ActionPanelItem],
    ) -> (Task<crate::Message>, Option<ActionHandler>) {
        match message {
            Message::Open => {
                self.visible = true;
                self.selected_index = 0;
                self.animation.start_time = Some(Instant::now());
                self.animation.opacity = animation::OPACITY_START;
                self.animation.scale = animation::SCALE_START;
                (iced::widget::operation::focus(self.input_id.clone()), None)
            }
            Message::Close => {
                self.visible = false;
                self.reset();
                (Task::none(), None)
            }
            Message::InvokeAction(handler) => {
                self.visible = false;
                self.reset();
                (Task::none(), Some(handler))
            }
            Message::SearchChanged(text) => {
                self.search_text = text;
                self.selected_index = 0;
                (Task::none(), None)
            }
            Message::Select(index) => {
                self.selected_index = index;
                (Task::none(), None)
            }
            Message::MoveUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                (Task::none(), None)
            }
            Message::MoveDown => {
                let count = count_actions(actions, &self.search_text);
                if self.selected_index < count.saturating_sub(1) {
                    self.selected_index += 1;
                }
                (Task::none(), None)
            }
            Message::InvokeSelected => {
                let filtered = filter_actions(actions, &self.search_text);
                let action = filtered
                    .iter()
                    .flat_map(|item| match item {
                        ActionPanelItem::Action(a) => std::slice::from_ref(a).iter(),
                        ActionPanelItem::Section(s) => s.children.iter(),
                    })
                    .nth(self.selected_index);

                if let Some(action) = action {
                    if let Some(handler) = &action.handler {
                        self.visible = false;
                        self.reset();
                        return (Task::none(), Some(handler.clone()));
                    }
                }
                (Task::none(), None)
            }
            Message::Tick(now) => {
                if let Some(start) = self.animation.start_time {
                    let elapsed = now.duration_since(start).as_millis() as f32;
                    let duration = animation::DURATION_MS as f32;
                    let t = (elapsed / duration).clamp(0.0, 1.0);

                    let ease = 1.0 - (1.0 - t).powi(2);

                    self.animation.opacity =
                        animation::OPACITY_START + (1.0 - animation::OPACITY_START) * ease;
                    self.animation.scale =
                        animation::SCALE_START + (1.0 - animation::SCALE_START) * ease;

                    if t >= 1.0 {
                        self.animation.start_time = None;
                        self.animation.opacity = 1.0;
                        self.animation.scale = 1.0;
                    }
                }
                (Task::none(), None)
            }
        }
    }

    fn reset(&mut self) {
        self.search_text.clear();
        self.selected_index = 0;
        self.animation.start_time = None;
        self.animation.opacity = 1.0;
        self.animation.scale = 1.0;
    }
}

pub fn render_action_panel<'a>(
    state: &'a State,
    actions: &[ActionPanelItem],
) -> Element<'a, Message> {
    let filtered_actions = filter_actions(actions, &state.search_text);
    let search_text_owned = state.search_text.to_string();
    let filtered_count = filtered_actions.len();

    let text_color = Color {
        a: state.animation.opacity,
        ..TEXT_COLOR
    };
    let section_title_color = Color {
        a: 0.5 * state.animation.opacity,
        ..SECTION_TITLE_COLOR
    };
    let bg_color = Color {
        a: state.animation.opacity,
        ..color!(0x2c2c2c)
    };

    let mut current_index = 0usize;

    let actions_col = filtered_actions.into_iter().enumerate().fold(
        column![].width(Length::Fill).padding([12, 0]),
        |col, (idx, action)| match action {
            ActionPanelItem::Action(action) => {
                let is_selected = current_index == state.selected_index;
                current_index += 1;

                col.push(
                    container(render_action_owned(
                        action,
                        is_selected,
                        current_index - 1,
                        state.animation.opacity,
                    ))
                    .width(Length::Fill)
                    .padding([0, 8]),
                )
            }
            ActionPanelItem::Section(section) => {
                let is_first = idx == 0;

                let mut col = col.push(render_section_owned(
                    section,
                    state.selected_index,
                    &mut current_index,
                    is_first,
                    section_title_color,
                    state.animation.opacity,
                ));

                if idx < filtered_count - 1 {
                    col = col.push(rule::horizontal(1).style(|theme| rule::Style {
                        color: color!(0x404040),
                        ..rule::default(theme)
                    }));
                }

                col
            }
        },
    );

    let search_bar = text_input("Search for actions...", &search_text_owned)
        .id(state.input_id.clone())
        .on_input(Message::SearchChanged)
        .size(13)
        .padding([0, 16])
        .style(move |_theme: &Theme, _status| text_input::Style {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: iced::Border::default(),
            icon: text_color,
            placeholder: Color::from_rgba(1.0, 1.0, 1.0, 0.4 * state.animation.opacity),
            value: text_color,
            selection: Color::from_rgba(1.0, 1.0, 1.0, 0.2 * state.animation.opacity),
        });

    let search_container = column![rule::horizontal(1), container(search_bar).center_y(44)];

    let action_panel = column![
        container(opaque(column![actions_col, search_container])).style(move |_theme| {
            container::Style {
                background: Some(bg_color.into()),
                border: Border::default().rounded(8.0),
                ..Default::default()
            }
        }),
        space().height(40)
    ]
    .width(Length::Fixed(368.0));

    opaque(
        mouse_area(
            container(action_panel)
                .align_bottom(Length::Fill)
                .align_right(Length::Fill)
                .padding([0, 12]),
        )
        .on_press(Message::Close),
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
    opacity: f32,
) -> Element<'static, Message> {
    let title = action.title.clone();
    let icon_char = action
        .icon
        .as_ref()
        .and_then(|icon_name| icons::get_icon(icon_name))
        .map(|s| s.to_string());

    let text_color = Color {
        a: opacity,
        ..TEXT_COLOR
    };

    let content: Element<'static, Message> = if let Some(icon) = icon_char {
        row![
            text(icon).font(ICON_FONT).size(18).color(text_color),
            text(title).font(INTER_FONT).color(text_color)
        ]
        .align_y(Center)
        .height(40)
        .spacing(10)
        .into()
    } else {
        container(text(title).font(INTER_FONT).color(text_color))
            .height(40)
            .align_y(Center)
            .into()
    };

    let selected_bg = Color {
        a: opacity,
        ..SELECTED_BG
    };

    let mut btn = Button::new(content)
        .width(Length::Fill)
        .padding([0, 10])
        .style(move |_theme, _status| {
            if is_selected {
                button::Style {
                    background: Some(selected_bg.into()),
                    text_color,
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                }
            } else {
                button::Style {
                    background: None,
                    text_color,
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                }
            }
        });

    if let Some(handler) = action.handler {
        btn = btn.on_press(Message::InvokeAction(handler));
    } else {
        btn = btn.on_press(Message::Select(index));
    }

    btn.into()
}

fn render_section_owned(
    section: ActionPanelSection,
    selected_index: usize,
    current_index: &mut usize,
    is_first: bool,
    section_title_color: Color,
    opacity: f32,
) -> Element<'static, Message> {
    let section_title = text(section.props.title.clone())
        .size(13)
        .line_height(LineHeight::Absolute(iced::Pixels(14.0)))
        .color(section_title_color)
        .font(INTER_FONT);

    let section_actions = section
        .children
        .into_iter()
        .fold(column![].spacing(0), |col, action| {
            let is_selected = *current_index == selected_index;
            let idx = *current_index;
            *current_index += 1;
            col.push(render_action_owned(action, is_selected, idx, opacity))
        });

    let top_pad = if is_first { 2.0 } else { 16.0 };

    column![
        container(section_title).padding(Padding::from(0).left(10)),
        section_actions
    ]
    .spacing(10)
    .padding(Padding::from(8).top(top_pad))
    .into()
}
