use chrono::{Local, TimeZone};
use iced::gradient::Linear;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::scrollable::Viewport;
use iced::widget::text::LineHeight;
use iced::widget::{column, container, image, mouse_area, row, scrollable, space, stack, text};
use iced::{Alignment, Background, Color, Element, Length, Padding, Task};
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
    preview_viewport: Option<Viewport>,
    preview_scrollable_id: iced::widget::Id,
    last_click: Option<(usize, Instant)>,
    current_actions: ActionPanel,
    hovered_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum ClipboardHistoryMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
    PreviewScrolled(Viewport),
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
            preview_viewport: None,
            preview_scrollable_id: iced::widget::Id::unique(),
            last_click: None,
            current_actions: ActionPanel::default(),
            hovered_index: None,
        };
        screen.update_actions();
        screen
    }

    fn create_state(items: &Vec<ClipboardEntry>) -> SelectionState<ClipboardEntry> {
        let now = Local::now();
        let today = now.date_naive();

        let mut buckets: Vec<(String, Vec<ClipboardEntry>)> = vec![
            ("Today".to_string(), Vec::new()),
            ("Yesterday".to_string(), Vec::new()),
            ("This Week".to_string(), Vec::new()),
            ("This Month".to_string(), Vec::new()),
            ("This Year".to_string(), Vec::new()),
            ("Older".to_string(), Vec::new()),
        ];

        for item in items {
            if let Some(item_dt) = Local.timestamp_opt(item.timestamp as i64, 0).latest() {
                let item_date = item_dt.date_naive();
                let diff = today.signed_duration_since(item_date).num_days();

                let bucket_idx = if diff < 0 {
                    0
                } else if diff == 0 {
                    0
                } else if diff == 1 {
                    1
                } else if diff < 7 {
                    2
                } else if diff < 30 {
                    3
                } else if diff < 365 {
                    4
                } else {
                    5
                };

                buckets[bucket_idx].1.push(item.clone());
            } else {
                buckets[5].1.push(item.clone());
            }
        }

        let sections = buckets
            .into_iter()
            .filter(|(_, items)| !items.is_empty())
            .map(|(title, items)| Section {
                title,
                items,
                columns: Some(1),
            })
            .collect();

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

                        self.preview_viewport = None;
                        return Task::batch(vec![
                            self.scroll_to_selection(dir),
                            iced::widget::operation::snap_to(
                                self.preview_scrollable_id.clone(),
                                iced::widget::scrollable::RelativeOffset::START,
                            ),
                        ]);
                    }
                }
                Task::none()
            }
            ClipboardHistoryMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
            ClipboardHistoryMessage::PreviewScrolled(viewport) => {
                self.preview_viewport = Some(viewport);
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

                    self.preview_viewport = None;
                    return Task::batch(vec![
                        self.scroll_to_selection(0),
                        iced::widget::operation::snap_to(
                            self.preview_scrollable_id.clone(),
                            iced::widget::scrollable::RelativeOffset::START,
                        ),
                    ]);
                }
                Task::none()
            }
            ClipboardHistoryMessage::RunAction(_) => Task::none(),
        }
    }

    fn scroll_to_selection(&self, direction: i32) -> Task<ClipboardHistoryMessage> {
        let (layout_index, header_index) =
            match self.state.get_layout_index(HeaderPolicy::IfTitleNotEmpty) {
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

        let mut item_cursor = 0;

        let list_col = self
            .state
            .sections
            .iter()
            .flat_map(|section| {
                let mut elements = Vec::new();

                if !section.title.is_empty() {
                    elements.push(
                        container(text(&section.title).size(12).color(secondary_text_color))
                            .padding(Padding::from(4).top(12))
                            .width(Length::Fill)
                            .into(),
                    );
                }

                let cursor_pos = item_cursor;
                let items = section.items.iter().enumerate().map(|(i, entry)| {
                    let global_index = cursor_pos + i;
                    let is_selected = global_index == self.state.selected_index;
                    let is_hovered = self.hovered_index == Some(global_index);

                    let (icon, preview_text, is_truncated) = match &entry.content {
                        ClipboardContent::Text(t) => {
                            let trimmed = t.trim();
                            let mut lines = trimmed.lines();
                            let first_line = lines.next().unwrap_or("");
                            let has_multiline = lines.next().is_some();

                            let max_chars = 100; // we truncate to 100 chars here as a heuristic, even though we're clipping
                            let char_count = first_line.chars().count();

                            let (display_text, text_was_cut) = if char_count > max_chars {
                                (first_line.chars().take(max_chars).collect::<String>(), true)
                            } else {
                                (first_line.to_string(), false)
                            };

                            let display_text = if display_text.is_empty() {
                                "Empty text".to_string()
                            } else {
                                display_text
                            };

                            ("📄", display_text, has_multiline || text_was_cut)
                        }
                        ClipboardContent::Image(_) => ("🖼️", "Image".to_string(), false),
                    };

                    let text_widget = text(preview_text)
                        .wrapping(text::Wrapping::None)
                        .size(14)
                        .color(if is_selected {
                            text_color
                        } else {
                            secondary_text_color
                        })
                        .width(Length::Fill)
                        .shaping(iced::widget::text::Shaping::Advanced);

                    let content_element: Element<ClipboardHistoryMessage> = if is_truncated {
                        let fade_to_color = if is_selected {
                            selection_color
                        } else if is_hovered {
                            hover_color
                        } else {
                            theme.colors.background
                        };

                        stack![
                            row![text(icon).size(16), text_widget]
                                .spacing(10)
                                .clip(true)
                                .align_y(Alignment::Center),
                            row![
                                space().width(Length::Fill),
                                container(space()).width(100).height(Length::Fill).style(
                                    move |_| container::Style {
                                        background: Some(Background::Gradient(
                                            iced::Gradient::Linear(
                                                Linear::new(1.571)
                                                    .add_stop(0.0, Color::TRANSPARENT)
                                                    .add_stop(1.0, fade_to_color)
                                            )
                                        )),
                                        ..Default::default()
                                    }
                                )
                            ]
                        ]
                        .into()
                    } else {
                        row![text(icon).size(16), text_widget]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .into()
                    };

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

                    let item_container = container(content_element)
                        .width(Length::Fill)
                        .padding(8)
                        .style(container_style);

                    let item_area = mouse_area(item_container)
                        .on_press(ClipboardHistoryMessage::ItemClicked(global_index))
                        .on_enter(ClipboardHistoryMessage::ItemHovered(global_index))
                        .on_exit(ClipboardHistoryMessage::ItemUnhovered(global_index));

                    item_area.into()
                });

                item_cursor += section.items.len();
                elements.extend(items);
                elements
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
                    let trimmed = t.trim();
                    if trimmed.starts_with('#') && (trimmed.len() == 4 || trimmed.len() == 7) {
                        let color = parse_hex_color(trimmed);
                        let size = 76.0;
                        let border_width = 4.0;

                        container(
                            column![
                                container(
                                    space()
                                        .width(size + border_width * 2.0)
                                        .height(size + border_width * 2.0)
                                )
                                .style(move |_| {
                                    container::Style {
                                        background: Some(color.into()),
                                        border: iced::Border {
                                            radius: (size / 2.0 + border_width).into(),
                                            width: border_width,
                                            color: Color { a: 0.4, ..color },
                                        },
                                        ..Default::default()
                                    }
                                }),
                                text(trimmed).size(15).color(text_color)
                            ]
                            .spacing(6)
                            .align_x(Alignment::Center),
                        )
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                    } else {
                        let lines: Vec<&str> = t.lines().collect();
                        if lines.len() > 500 {
                            let row_height = LineHeight::default().to_absolute(14.0.into()).0;
                            let vp = self.preview_viewport.as_ref();
                            let offset = vp.map(|v| v.absolute_offset().y).unwrap_or(0.0);
                            let height = vp.map(|v| v.bounds().height).unwrap_or(500.0);

                            let start_index = (offset / row_height).floor() as usize;
                            let visible_count = (height / row_height).ceil() as usize + 5;
                            let end_index = (start_index + visible_count).min(lines.len());

                            let top_spacer =
                                space().height(Length::Fixed(start_index as f32 * row_height));
                            let bottom_spacer = space().height(Length::Fixed(
                                (lines.len().saturating_sub(end_index)) as f32 * row_height,
                            ));

                            let visible_lines: Vec<Element<_>> = lines
                                .iter()
                                .skip(start_index)
                                .take(end_index - start_index)
                                .map(|line| {
                                    text(line.to_string())
                                        .size(14)
                                        .height(Length::Fixed(row_height))
                                        .wrapping(iced::widget::text::Wrapping::None)
                                        .color(text_color)
                                        .into()
                                })
                                .collect();

                            let content = column![top_spacer]
                                .extend(visible_lines)
                                .push(bottom_spacer);

                            container(
                                scrollable(content)
                                    .id(self.preview_scrollable_id.clone())
                                    .on_scroll(ClipboardHistoryMessage::PreviewScrolled)
                                    .width(Length::Fill),
                            )
                            .height(Length::Fill)
                            .width(Length::Fill)
                        } else {
                            container(
                                scrollable(text(t).size(14).color(text_color))
                                    .id(self.preview_scrollable_id.clone())
                                    .width(Length::Fill),
                            )
                            .height(Length::Fill)
                            .width(Length::Fill)
                        }
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

            let date_str = if let Some(dt) = Local.timestamp_opt(entry.timestamp as i64, 0).latest()
            {
                dt.format("%Y-%m-%d %H:%M").to_string()
            } else {
                "Unknown".to_string()
            };

            let metadata_section = container(
                column![
                    text("Information").size(12).color(secondary_text_color),
                    metadata_row(
                        "Content type",
                        match &entry.content {
                            ClipboardContent::Text(_) => "Text".to_string(),
                            ClipboardContent::Image(_) => "Image".to_string(),
                        },
                        theme
                    ),
                    metadata_row("Last copied", date_str, theme),
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
    value: String,
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
