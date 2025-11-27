use iced::widget::scrollable::Viewport;
use iced::widget::{self, column, container, row, scrollable, text};
use iced::{
    Alignment, Element, Length, Task,
    keyboard::{Key, Modifiers, key::Named},
    widget::operation,
};

use crate::apps::AppEntry;
use crate::extensions::ExtensionCommand;
use crate::screens::Shell;

#[derive(Clone, Debug)]
pub enum RootItem {
    Extension(ExtensionCommand),
    App(AppEntry),
}

pub struct RootScreen {
    items: Vec<RootItem>,
    filtered_items: Vec<RootItem>,
    selected_index: usize,
    viewport: Option<Viewport>,
    scrollable_id: widget::Id,
    #[cfg(feature = "soulver")]
    calculator_result: Option<String>,
}

#[derive(Clone, Debug)]
pub enum RootMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
}

impl RootScreen {
    pub fn new(commands: Vec<ExtensionCommand>, apps: Vec<AppEntry>) -> Self {
        let mut items = Vec::new();
        for cmd in commands {
            items.push(RootItem::Extension(cmd));
        }
        for app in apps {
            items.push(RootItem::App(app));
        }

        Self {
            filtered_items: items.clone(),
            items,
            selected_index: 0,
            viewport: None,
            scrollable_id: widget::Id::unique(),
            #[cfg(feature = "soulver")]
            calculator_result: None,
        }
    }

    pub fn update(&mut self, message: RootMessage) -> Task<RootMessage> {
        match message {
            RootMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    let moved = match named_key {
                        Named::ArrowDown => {
                            self.select_next();
                            true
                        }
                        Named::ArrowUp => {
                            self.select_prev();
                            true
                        }
                        _ => false,
                    };
                    if moved {
                        return self.scroll_to_selection();
                    }
                }
                Task::none()
            }
            RootMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, RootMessage> {
        let mut ui_rows: Vec<Element<'_, RootMessage>> = Vec::new();

        #[cfg(feature = "soulver")]
        if let Some(result) = &self.calculator_result {
            let calc_item = container(
                row![
                    column![
                        text(result).size(14),
                        text("Calculator")
                            .size(12)
                            .color(iced::Color::from_rgb8(0x88, 0x88, 0x88)),
                    ]
                    .spacing(2),
                ]
                .align_y(Alignment::Center)
                .padding(12),
            )
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(
                    0x33, 0x33, 0x55,
                ))),
                ..Default::default()
            })
            .width(Length::Fill);
            ui_rows.push(calc_item.into());
        }

        for (idx, item) in self.filtered_items.iter().enumerate() {
            let is_selected = idx == self.selected_index;
            let background = if is_selected {
                iced::Color::from_rgb8(0x44, 0x44, 0x44)
            } else {
                iced::Color::TRANSPARENT
            };

            let (title, subtitle) = match item {
                RootItem::Extension(cmd) => {
                    let sub = cmd
                        .command_subtitle
                        .as_ref()
                        .unwrap_or(&cmd.extension_title);
                    (cmd.command_title.clone(), sub.clone())
                }
                RootItem::App(app) => (app.name.clone(), "Application".to_string()),
            };

            let item_row = container(
                row![
                    column![
                        text(title).size(14),
                        text(subtitle)
                            .size(12)
                            .color(iced::Color::from_rgb8(0x88, 0x88, 0x88)),
                    ]
                    .spacing(2),
                ]
                .align_y(Alignment::Center)
                .padding(12),
            )
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(background)),
                ..Default::default()
            })
            .width(Length::Fill);
            ui_rows.push(item_row.into());
        }

        let content = column(ui_rows).width(Length::Fill);

        scrollable(content)
            .id(self.scrollable_id.clone())
            .on_scroll(RootMessage::Scrolled)
            .height(Length::Fill)
            .into()
    }

    pub fn get_selected_item(&self) -> Option<&RootItem> {
        self.filtered_items.get(self.selected_index)
    }

    fn scroll_to_selection(&self) -> Task<RootMessage> {
        let item_height = 60.0;
        let target_y = self.selected_index as f32 * item_height;

        let offset = match &self.viewport {
            Some(vp) => {
                let view_top = vp.absolute_offset().y;
                let view_bottom = view_top + vp.bounds().height;
                let target_top = target_y;
                let target_bottom = target_top + item_height;

                if target_top < view_top {
                    Some(target_top)
                } else if target_bottom > view_bottom {
                    Some(target_bottom - vp.bounds().height)
                } else {
                    None
                }
            }
            None => None,
        };

        match offset {
            Some(y) => operation::scroll_to(
                self.scrollable_id.clone(),
                scrollable::AbsoluteOffset { x: 0.0, y },
            ),
            None => Task::none(),
        }
    }

    fn select_next(&mut self) {
        let total = self.filtered_items.len();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
        }
    }

    fn select_prev(&mut self) {
        let total = self.filtered_items.len();
        if total > 0 {
            self.selected_index = (self.selected_index + total - 1) % total;
        }
    }
}

impl Shell for RootScreen {
    fn can_search(&self) -> bool {
        true
    }

    fn on_search(&mut self, query: &str) {
        let query_lower = query.to_lowercase();

        #[cfg(feature = "soulver")]
        {
            self.calculator_result = if query.is_empty() {
                None
            } else {
                crate::soulver::calculate(query).and_then(|r| {
                    if r.result_type == "none" || r.value.is_empty() {
                        None
                    } else {
                        Some(r.value)
                    }
                })
            };
        }

        if query.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| match item {
                    RootItem::Extension(cmd) => {
                        cmd.command_title.to_lowercase().contains(&query_lower)
                            || cmd.extension_title.to_lowercase().contains(&query_lower)
                            || cmd
                                .command_subtitle
                                .as_ref()
                                .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    }
                    RootItem::App(app) => {
                        app.name.to_lowercase().contains(&query_lower)
                            || app.id.to_lowercase().contains(&query_lower)
                    }
                })
                .cloned()
                .collect();
        }

        self.selected_index = 0;
    }

    fn get_action_panel(&mut self) -> Option<&mut crate::components::actions::ActionPanel> {
        None
    }
}
