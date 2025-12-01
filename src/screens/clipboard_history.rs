use chrono::{Local, TimeZone};
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::scrollable::Viewport;
use iced::widget::{container, row, scrollable, text};
use iced::{Color, Element, Length, Padding, Task};
use std::time::{Duration, Instant};

use crate::clipboard_history::{ClipboardContent, ClipboardEntry, get_history};
use crate::components::actions::{Action, ActionHandler, ActionPanel, ActionPanelItem};
use crate::components::clipboard::list_item::render_list_item;
use crate::components::clipboard::preview::render_preview_pane;
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
    has_application_metadata: bool,
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
        let has_application_metadata = history.iter().any(|entry| {
            entry
                .window_title
                .as_ref()
                .map(|title| !title.trim().is_empty())
                .unwrap_or(false)
        });

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
            has_application_metadata,
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
        let secondary_text_color = theme.colors.text_60;
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

                    render_list_item(entry, global_index, is_selected, is_hovered, theme)
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

        let right_pane_content = render_preview_pane(
            self.state.selected_item(),
            theme,
            self.preview_viewport.as_ref(),
            &self.preview_scrollable_id,
            self.has_application_metadata,
        );

        let right_pane = container(right_pane_content)
            .width(Length::FillPortion(2))
            .height(Length::Fill);

        row![left_pane, right_pane].into()
    }
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
