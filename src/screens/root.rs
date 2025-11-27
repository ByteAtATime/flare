use iced::widget::scrollable::Viewport;
use iced::widget::{self, container, image, row, scrollable, svg, text};
use iced::{
    Alignment, Element, Length, Task,
    keyboard::{Key, Modifiers, key::Named},
};
use std::path::PathBuf;

use crate::apps::AppEntry;
use crate::components::actions::{Action, ActionHandler, ActionPanel, ActionPanelItem};
use crate::components::column::Column;
use crate::extensions::ExtensionCommand;
use crate::frecency::FrecencyStore;
use crate::globals::POSITION_TRACKER;
use crate::message::Message;
use crate::screens::Shell;
use crate::selection::{HeaderPolicy, Section, SelectionState, scroll_to};

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

#[derive(Clone, Debug)]
pub enum ResolvedIcon {
    FontChar(String),
    Svg(PathBuf),
    Image(PathBuf),
}

#[derive(Clone, Debug)]
pub struct RootItem {
    pub id: String,
    pub kind: RootItemKind,
    pub actions: ActionPanel,
    pub resolved_icon: ResolvedIcon,
}

#[derive(Clone, Debug)]
pub enum RootItemKind {
    Extension(ExtensionCommand),
    App(AppEntry),
}

pub struct RootScreen {
    items: Vec<RootItem>,
    filtered_items: Vec<RootItem>,
    state: SelectionState<RootItem>,
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
        let mut items = Vec::with_capacity(commands.len() + apps.len());

        for cmd in commands {
            let icon = resolve_extension_icon(&cmd);
            let id = format!("ext:{}:{}", cmd.extension_name, cmd.command_name);
            items.push(RootItem {
                id,
                actions: create_action_panel(&RootItemKind::Extension(cmd.clone())),
                kind: RootItemKind::Extension(cmd),
                resolved_icon: icon,
            });
        }

        for app in apps {
            let icon = resolve_app_icon(&app);
            let id = format!("app:{}", app.id);
            items.push(RootItem {
                id,
                actions: create_action_panel(&RootItemKind::App(app.clone())),
                kind: RootItemKind::App(app),
                resolved_icon: icon,
            });
        }

        let state = Self::create_state(&items);

        Self {
            filtered_items: items.clone(),
            items,
            state,
            viewport: None,
            scrollable_id: widget::Id::unique(),
            #[cfg(feature = "soulver")]
            calculator_result: None,
        }
    }

    pub fn sort_items(&mut self, store: &FrecencyStore) {
        store.sort(&mut self.items, |item| &item.id);
        self.filtered_items = self.items.clone();
        self.state = Self::create_state(&self.filtered_items);
    }

    fn create_state(items: &Vec<RootItem>) -> SelectionState<RootItem> {
        let sections = vec![Section {
            title: String::new(),
            items: items.clone(),
            columns: Some(1),
        }];
        SelectionState::new(sections, 1)
    }

    pub fn update(&mut self, message: RootMessage) -> Task<RootMessage> {
        match message {
            RootMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    let moved = match named_key {
                        Named::ArrowDown => {
                            self.state.move_vertical(1);
                            true
                        }
                        Named::ArrowUp => {
                            self.state.move_vertical(-1);
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
        let mut ui_rows: Vec<Element<'_, RootMessage>> =
            Vec::with_capacity(self.filtered_items.len() + 1);

        #[cfg(feature = "soulver")]
        let has_calc = self.calculator_result.is_some();
        #[cfg(not(feature = "soulver"))]
        let has_calc = false;

        #[cfg(feature = "soulver")]
        if let Some(result) = &self.calculator_result {
            let calc_item = container(
                row![
                    text(result),
                    widget::space().width(Length::Fill),
                    text("Calculator").color(iced::Color::from_rgb8(0x88, 0x88, 0x88)),
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

        let visible_range = self.viewport.as_ref().map(|vp| {
            let y = vp.absolute_offset().y;
            let height = vp.bounds().height;
            (y - 500.0, y + height + 500.0)
        });

        // Use global layout cache directly for rendering visibility optimization
        let layout_cache = crate::globals::LAYOUT_CACHE
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        for (idx, item) in self.filtered_items.iter().enumerate() {
            let is_selected = idx == self.state.selected_index;
            let background = if is_selected {
                iced::Color::from_rgb8(0x44, 0x44, 0x44)
            } else {
                iced::Color::TRANSPARENT
            };

            let layout_idx = idx + if has_calc { 1 } else { 0 };

            let is_visible = if let Some((start, end)) = visible_range {
                if let Some(bounds) = layout_cache.get(&layout_idx) {
                    let item_top = bounds.y;
                    let item_bottom = bounds.y + bounds.height;
                    item_bottom >= start && item_top <= end
                } else {
                    layout_idx < 30
                }
            } else {
                layout_idx < 30
            };

            let (title, subtitle, accessory) = match &item.kind {
                RootItemKind::Extension(cmd) => {
                    let sub = cmd
                        .command_subtitle
                        .clone()
                        .or_else(|| Some(cmd.extension_title.clone()));
                    (cmd.command_title.clone(), sub, "Command")
                }
                RootItemKind::App(app) => (app.name.clone(), None, "Application"),
            };

            let icon_element: Element<'_, RootMessage> = if is_visible {
                match &item.resolved_icon {
                    ResolvedIcon::FontChar(c) => text(c)
                        .font(ICON_FONT)
                        .size(20)
                        .width(24)
                        .align_x(Alignment::Center)
                        .into(),
                    ResolvedIcon::Svg(path) => svg(path).width(24).height(24).into(),
                    ResolvedIcon::Image(path) => image(path).width(24).height(24).into(),
                }
            } else {
                widget::space().width(24).height(24).into()
            };

            let mut row_content = row![icon_element].align_y(Alignment::Center).spacing(12);

            let mut text_col = widget::column![text(title).size(14)].spacing(2);

            if let Some(sub) = subtitle {
                text_col = text_col.push(
                    text(sub)
                        .size(12)
                        .color(iced::Color::from_rgb8(0x88, 0x88, 0x88)),
                );
            }

            row_content = row_content.push(text_col);
            row_content = row_content.push(widget::space().width(Length::Fill));
            row_content = row_content.push(
                text(accessory)
                    .size(12)
                    .color(iced::Color::from_rgb8(0x88, 0x88, 0x88)),
            );

            let item_row = container(row_content.padding(12))
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(background)),
                    ..Default::default()
                })
                .width(Length::Fill);

            ui_rows.push(item_row.into());
        }

        let content = Column::with_children(ui_rows)
            .width(Length::Fill)
            .id(POSITION_TRACKER.clone());

        scrollable(content)
            .id(self.scrollable_id.clone())
            .on_scroll(RootMessage::Scrolled)
            .height(Length::Fill)
            .into()
    }

    fn scroll_to_selection(&self) -> Task<RootMessage> {
        #[cfg(feature = "soulver")]
        let has_calc = self.calculator_result.is_some();
        #[cfg(not(feature = "soulver"))]
        let has_calc = false;

        let base_index = match self.state.get_layout_index(HeaderPolicy::Never) {
            Some(idx) => idx,
            None => return Task::none(),
        };

        let final_index = base_index + if has_calc { 1 } else { 0 };

        scroll_to(
            self.scrollable_id.clone(),
            self.viewport.as_ref(),
            final_index,
        )
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
                .filter(|item| match &item.kind {
                    RootItemKind::Extension(cmd) => {
                        cmd.command_title.to_lowercase().contains(&query_lower)
                            || cmd.extension_title.to_lowercase().contains(&query_lower)
                            || cmd
                                .command_subtitle
                                .as_ref()
                                .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    }
                    RootItemKind::App(app) => {
                        app.name.to_lowercase().contains(&query_lower)
                            || app.id.to_lowercase().contains(&query_lower)
                    }
                })
                .cloned()
                .collect();
        }

        self.state = Self::create_state(&self.filtered_items);
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        self.state.selected_item_mut().map(|item| &mut item.actions)
    }
}

fn create_action_panel(kind: &RootItemKind) -> ActionPanel {
    let handler = match kind {
        RootItemKind::Extension(cmd) => {
            let cmd = cmd.clone();
            ActionHandler::new(move || Task::done(Message::LaunchCommand(cmd.clone())))
        }
        RootItemKind::App(app) => {
            let app = app.clone();
            ActionHandler::new(move || Task::done(Message::LaunchApp(app.clone())))
        }
    };

    let action = Action {
        title: "Open".to_string(),
        icon: None,
        handler: Some(handler),
    };

    ActionPanel {
        children: vec![ActionPanelItem::Action(action)],
    }
}

fn resolve_app_icon(app: &AppEntry) -> ResolvedIcon {
    if std::path::Path::new(&app.icon).is_absolute() {
        let path = std::path::PathBuf::from(&app.icon);
        if path.extension().map_or(false, |e| e == "svg") {
            ResolvedIcon::Svg(path)
        } else {
            ResolvedIcon::Image(path)
        }
    } else {
        if let Some(icon_path) = freedesktop_icons::lookup(&app.icon).find() {
            let path = std::path::PathBuf::from(icon_path);
            if path.extension().map_or(false, |e| e == "svg") {
                ResolvedIcon::Svg(path)
            } else {
                ResolvedIcon::Image(path)
            }
        } else {
            if let Some(c) = crate::icons::get_icon("app-window-16") {
                ResolvedIcon::FontChar(c.to_string())
            } else {
                ResolvedIcon::FontChar("".to_string())
            }
        }
    }
}

fn resolve_extension_icon(cmd: &ExtensionCommand) -> ResolvedIcon {
    let icon_str = cmd.command_icon.as_ref().or(cmd.extension_icon.as_ref());
    if let Some(s) = icon_str {
        if let Some(c) = crate::icons::get_icon(s) {
            return ResolvedIcon::FontChar(c.to_string());
        }

        let path = if std::path::Path::new(s).is_absolute() {
            std::path::PathBuf::from(s)
        } else {
            let assets_path = cmd.extension_path.join("assets").join(s);
            if assets_path.exists() {
                assets_path
            } else {
                cmd.extension_path.join(s)
            }
        };

        if path.extension().map_or(false, |e| e == "svg") {
            ResolvedIcon::Svg(path)
        } else {
            ResolvedIcon::Image(path)
        }
    } else {
        if let Some(c) = crate::icons::get_icon("box-16") {
            ResolvedIcon::FontChar(c.to_string())
        } else {
            ResolvedIcon::FontChar("".to_string())
        }
    }
}
