use iced::widget::{button, column, container, image, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length, Task};

use crate::clipboard_history::{ClipboardContent, ClipboardEntry, get_history};
use crate::components::actions::ActionPanel;
use crate::components::types::parse_hex_color;
use crate::screens::Shell;
use crate::theme::Theme;

pub struct ClipboardHistoryScreen {
    history: Vec<ClipboardEntry>,
    filtered_history: Vec<ClipboardEntry>,
    selected_index: usize,
}

#[derive(Clone, Debug)]
pub enum ClipboardHistoryMessage {
    ItemSelected(usize),
}

impl ClipboardHistoryScreen {
    pub fn new() -> Self {
        let history = get_history();
        Self {
            filtered_history: history.clone(),
            history,
            selected_index: 0,
        }
    }

    pub fn update(&mut self, message: ClipboardHistoryMessage) -> Task<ClipboardHistoryMessage> {
        match message {
            ClipboardHistoryMessage::ItemSelected(index) => {
                self.selected_index = index;
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, ClipboardHistoryMessage> {
        let text_color = theme.colors.text;
        let secondary_text_color = theme.colors.text_60;

        let list_col = self.filtered_history.iter().enumerate().fold(
            column![].spacing(2),
            |col, (i, entry)| {
                let is_selected = i == self.selected_index;

                let (icon, preview_text) = match &entry.content {
                    ClipboardContent::Text(t) => {
                        let line = t.lines().next().unwrap_or("").trim();
                        let text = if line.is_empty() { "Empty text" } else { line };
                        // TODO: use proper icons
                        ("📄", text)
                    }
                    ClipboardContent::Image(_) => ("🖼️", "Image"),
                };

                let content = row![
                    text(icon).size(16),
                    text(preview_text)
                        .size(14)
                        .color(if is_selected {
                            theme.colors.text
                        } else {
                            text_color
                        })
                        .width(Length::Fill)
                        .shaping(iced::widget::text::Shaping::Advanced)
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                let btn = button(content)
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(ClipboardHistoryMessage::ItemSelected(i))
                    .style(move |_, _| {
                        if is_selected {
                            iced::widget::button::Style {
                                background: Some(theme.colors.selection.into()),
                                text_color: theme.colors.text,
                                border: iced::Border::default().rounded(4),
                                ..Default::default()
                            }
                        } else {
                            iced::widget::button::Style {
                                background: None,
                                text_color,
                                ..Default::default()
                            }
                        }
                    });

                col.push(btn)
            },
        );

        let left_pane = container(scrollable(list_col))
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

        let right_pane_content = if let Some(entry) = self.filtered_history.get(self.selected_index)
        {
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
        if query.is_empty() {
            self.filtered_history = self.history.clone();
        } else {
            let q = query.to_lowercase();
            self.filtered_history = self
                .history
                .iter()
                .filter(|e| match &e.content {
                    ClipboardContent::Text(t) => t.to_lowercase().contains(&q),
                    ClipboardContent::Image(_) => "image".contains(&q),
                })
                .cloned()
                .collect();
        }
        self.selected_index = 0;
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        None
    }
}
