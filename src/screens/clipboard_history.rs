use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::scrollable::Viewport;
use iced::widget::{column, container, image, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length, Task};
use std::time::{Duration, Instant};

use crate::clipboard_history::{ClipboardContent, ClipboardEntry, get_history};
use crate::components::actions::{Action, ActionHandler, ActionPanel, ActionPanelItem};
use crate::components::types::parse_hex_color;
use crate::message::Message;
use crate::screens::Shell;
use crate::selection::{HeaderPolicy, Section, SelectionState, scroll_to};
use crate::theme::Theme;

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);

pub struct ClipboardHistoryScreen {
    raw_history: Vec<ClipboardEntry>,
    state: SelectionState<ClipboardEntry>,
    viewport: Option<Viewport>,
    scrollable_id: iced::widget::Id,
    last_click: Option<(usize, Instant)>,
    current_actions: ActionPanel,
    hovered_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum ClipboardHistoryMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
    ItemClicked(usize),
    ItemHovered(usize),
    ItemUnhovered(usize),
    RunAction(ActionHandler),
}

impl ClipboardHistoryScreen {
    pub fn new() -> Self {
        let history = get_history();
        let state = Self::create_state(&history);

        let mut screen = Self {
            raw_history: history,
            state,
            viewport: None,
            scrollable_id: iced::widget::Id::unique(),
            last_click: None,
            current_actions: ActionPanel::default(),
            hovered_index: None,
        };
        screen.update_actions();
        screen
    }

    fn create_state(items: &Vec<ClipboardEntry>) -> SelectionState<ClipboardEntry> {
        let sections = vec![Section {
            title: String::new(),
            items: items.clone(),
            columns: Some(1),
        }];
        SelectionState::new(sections, 1)
    }

    fn update_actions(&mut self) {
        if let Some(entry) = self.state.selected_item() {
            self.current_actions = create_action_panel(entry);
        } else {
            self.current_actions = ActionPanel::default();
        }
    }

    pub fn update(&mut self, message: ClipboardHistoryMessage) -> Task<ClipboardHistoryMessage> {
        match message {
            ClipboardHistoryMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    let direction = match named_key {
                        Named::ArrowDown => {
                            self.state.move_vertical(1);
                            Some(1)
                        }
                        Named::ArrowUp => {
                            self.state.move_vertical(-1);
                            Some(-1)
                        }
                        _ => None,
                    };

                    if let Some(dir) = direction {
                        self.update_actions();
                        self.last_click = None;
                        return self.scroll_to_selection(dir);
                    }
                }
                Task::none()
            }
            ClipboardHistoryMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
            ClipboardHistoryMessage::ItemHovered(idx) => {
                self.hovered_index = Some(idx);
                Task::none()
            }
            ClipboardHistoryMessage::ItemUnhovered(idx) => {
                if self.hovered_index == Some(idx) {
                    self.hovered_index = None;
                }
                Task::none()
            }
            ClipboardHistoryMessage::ItemClicked(index) => {
                let now = Instant::now();
                let mut is_double_click = false;

                if let Some((last_idx, last_time)) = self.last_click {
                    if last_idx == index && now.duration_since(last_time) < DOUBLE_CLICK_THRESHOLD {
                        is_double_click = true;
                    }
                }

                if is_double_click {
                    self.last_click = None;
                    if let Some(action) = self.current_actions.children.first() {
                        if let ActionPanelItem::Action(a) = action {
                            if let Some(handler) = &a.handler {
                                return Task::done(ClipboardHistoryMessage::RunAction(
                                    handler.clone(),
                                ));
                            }
                        }
                    }
                } else {
                    self.last_click = Some((index, now));
                    self.state.selected_index = index;
                    self.update_actions();
                    return self.scroll_to_selection(0);
                }
                Task::none()
            }
            ClipboardHistoryMessage::RunAction(_) => Task::none(),
        }
    }

    fn scroll_to_selection(&self, direction: i32) -> Task<ClipboardHistoryMessage> {
        let (layout_index, header_index) = match self.state.get_layout_index(HeaderPolicy::Never) {
            Some(indices) => indices,
            None => return Task::none(),
        };

        scroll_to(
            self.scrollable_id.clone(),
            self.viewport.as_ref(),
            layout_index,
            header_index,
            direction,
        )
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, ClipboardHistoryMessage> {
        let text_color = theme.colors.text;
        let secondary_text_color = theme.colors.text_60;
        let selection_color = theme.colors.selection;
        let hover_color = Color::from_rgb8(39, 39, 39);

        let list_col = self
            .state
            .sections
            .iter()
            .flat_map(|section| {
                section.items.iter().enumerate().map(|(i, entry)| {
                    let global_index = i;
                    let is_selected = global_index == self.state.selected_index;
                    let is_hovered = self.hovered_index == Some(global_index);

                    let (icon, preview_text) = match &entry.content {
                        ClipboardContent::Text(t) => {
                            let line = t.lines().next().unwrap_or("").trim();
                            let text = if line.is_empty() { "Empty text" } else { line };
                            ("📄", text)
                        }
                        ClipboardContent::Image(_) => ("🖼️", "Image"),
                    };

                    let content = row![
                        text(icon).size(16),
                        text(preview_text)
                            .size(14)
                            .color(if is_selected {
                                text_color
                            } else {
                                secondary_text_color
                            })
                            .width(Length::Fill)
                            .shaping(iced::widget::text::Shaping::Advanced)
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center);

                    let container_style = move |_theme: &iced::Theme| container::Style {
                        background: Some(if is_selected {
                            selection_color.into()
                        } else if is_hovered {
                            hover_color.into()
                        } else {
                            Color::TRANSPARENT.into()
                        }),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let item_container = container(content)
                        .width(Length::Fill)
                        .padding(8)
                        .style(container_style);

                    let item_area = mouse_area(item_container)
                        .on_press(ClipboardHistoryMessage::ItemClicked(global_index))
                        .on_enter(ClipboardHistoryMessage::ItemHovered(global_index))
                        .on_exit(ClipboardHistoryMessage::ItemUnhovered(global_index));

                    item_area.into()
                })
            })
            .collect::<Vec<Element<'a, ClipboardHistoryMessage>>>();

        let list_content = crate::components::column::Column::with_children(list_col)
            .spacing(2)
            .id(crate::globals::POSITION_TRACKER.clone());

        let left_pane = container(
            scrollable(list_content)
                .id(self.scrollable_id.clone())
                .on_scroll(ClipboardHistoryMessage::Scrolled)
                .height(Length::Fill),
        )
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .padding(10)
        .style(|_| container::Style {
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        });

        let right_pane_content = if let Some(entry) = self.state.selected_item() {
            let preview = match &entry.content {
                ClipboardContent::Text(t) => {
                    let t = t.trim();
                    if t.starts_with('#') && (t.len() == 4 || t.len() == 7) {
                        let color = parse_hex_color(t);
                        container(
                            column![
                                container(space().width(64).height(64)).style(move |_| {
                                    container::Style {
                                        background: Some(color.into()),
                                        border: iced::Border {
                                            radius: 32.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                }),
                                text(t).size(20).color(text_color)
                            ]
                            .spacing(16)
                            .align_x(Alignment::Center),
                        )
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                    } else {
                        container(scrollable(text(t).size(14).color(text_color)))
                            .height(Length::Fill)
                            .width(Length::Fill)
                    }
                }
                ClipboardContent::Image(img) => {
                    let handle = iced::widget::image::Handle::from_bytes(img.bytes.clone());
                    container(image(handle).content_fit(iced::ContentFit::Contain))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                }
            };

            let preview_section = container(preview)
                .width(Length::Fill)
                .height(Length::FillPortion(3))
                .padding(20);

            let metadata_section = container(
                column![
                    text("Information").size(12).color(secondary_text_color),
                    metadata_row(
                        "Content type",
                        match &entry.content {
                            ClipboardContent::Text(_) => "Text",
                            ClipboardContent::Image(_) => "Image",
                        },
                        theme
                    ),
                    metadata_row("Last copied", "Today", theme), // TODO: format timestamp
                ]
                .spacing(8),
            )
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .padding(15)
            .style(|_| container::Style {
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });

            column![preview_section, metadata_section]
        } else {
            column![
                container(text("No item selected").color(secondary_text_color))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            ]
        };

        let right_pane = container(right_pane_content)
            .width(Length::FillPortion(2))
            .height(Length::Fill);

        row![left_pane, right_pane].into()
    }
}

fn metadata_row<'a>(
    label: &'a str,
    value: &'a str,
    theme: &'a Theme,
) -> Element<'a, ClipboardHistoryMessage> {
    row![
        text(label)
            .size(13)
            .color(theme.colors.text_60)
            .width(Length::Fill),
        text(value).size(13).color(theme.colors.text)
    ]
    .into()
}

impl Shell for ClipboardHistoryScreen {
    fn can_search(&self) -> bool {
        true
    }

    fn on_search(&mut self, query: &str) {
        let query = query.to_lowercase();
        let items: Vec<ClipboardEntry> = if query.is_empty() {
            self.raw_history.clone()
        } else {
            self.raw_history
                .iter()
                .filter(|e| match &e.content {
                    ClipboardContent::Text(t) => t.to_lowercase().contains(&query),
                    ClipboardContent::Image(_) => "image".contains(&query),
                })
                .cloned()
                .collect()
        };

        self.state = ClipboardHistoryScreen::create_state(&items);
        self.update_actions();
        self.hovered_index = None;
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        Some(&mut self.current_actions)
    }
}

fn create_action_panel(entry: &ClipboardEntry) -> ActionPanel {
    let content_for_copy = entry.content.clone();

    let copy_handler = ActionHandler::new(move || {
        let c = content_for_copy.clone();
        Task::perform(
            async move {
                let clipboard_content = match c {
                    ClipboardContent::Text(text) => crate::types::ClipboardContent::Text { text },
                    ClipboardContent::Image(_) => crate::types::ClipboardContent::Text {
                        text: String::new(),
                    },
                };
                let _ = crate::clipboard::copy(clipboard_content, false);
            },
            |_| Message::ToggleWindow,
        )
    });

    let copy_action = Action {
        title: "Copy to Clipboard".to_string(),
        icon: Some("clipboard-16".to_string()),
        handler: Some(copy_handler),
    };

    ActionPanel {
        children: vec![ActionPanelItem::Action(copy_action)],
    }
}
