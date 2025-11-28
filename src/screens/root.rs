use iced::widget::image::Handle as ImageHandle;
use iced::widget::scrollable::Viewport;
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{self, container, image, mouse_area, row, scrollable, space, svg, text};
use iced::{
    Alignment, Border, Color, Element, Length, Task,
    keyboard::{Key, Modifiers, key::Named},
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::apps::AppEntry;
use crate::components::actions::{Action, ActionHandler, ActionPanel, ActionPanelItem};
use crate::components::column::Column;
use crate::components::scrollable::scrollable_style;
use crate::extensions::ExtensionCommand;
use crate::frecency::FrecencyStore;
use crate::globals::POSITION_TRACKER;
use crate::message::Message;
use crate::screens::Shell;
use crate::selection::{HeaderPolicy, Section, SelectionState};
use crate::theme::Theme;

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");
const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);
const ROW_HEIGHT: f32 = 48.0;

#[derive(Clone, Debug)]
pub enum ResolvedIcon {
    FontChar(String),
    Svg(SvgHandle),
    Image(ImageHandle),
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
    last_click: Option<(usize, Instant)>,
    hovered_index: Option<usize>,
    #[cfg(feature = "soulver")]
    calculator_result: Option<String>,
}

#[derive(Clone, Debug)]
pub enum RootMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
    ItemClicked(usize),
    ItemHovered(usize),
    ItemUnhovered(usize),
    RunAction(ActionHandler),
}

impl RootScreen {
    pub fn new(commands: Vec<ExtensionCommand>, apps: Vec<AppEntry>) -> Self {
        let mut items = Vec::with_capacity(commands.len() + apps.len());

        for cmd in commands {
            let icon = resolve_extension_icon(&cmd);
            let id = format!("ext:{}:{}", cmd.extension_name, cmd.command_name);
            items.push(RootItem {
                id: id.clone(),
                actions: create_action_panel(&RootItemKind::Extension(cmd.clone()), &id),
                kind: RootItemKind::Extension(cmd),
                resolved_icon: icon,
            });
        }

        for app in apps {
            let icon = resolve_app_icon(&app);
            let id = format!("app:{}", app.id);
            items.push(RootItem {
                id: id.clone(),
                actions: create_action_panel(&RootItemKind::App(app.clone()), &id),
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
            last_click: None,
            hovered_index: None,
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
                        self.last_click = None;
                        return self.scroll_to_selection();
                    }
                }
                Task::none()
            }
            RootMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
            RootMessage::ItemHovered(idx) => {
                if self.hovered_index != Some(idx) {
                    self.hovered_index = Some(idx);
                }
                Task::none()
            }
            RootMessage::ItemUnhovered(idx) => {
                if self.hovered_index == Some(idx) {
                    self.hovered_index = None;
                }
                Task::none()
            }
            RootMessage::ItemClicked(index) => {
                let now = Instant::now();
                let mut is_double_click = false;

                if let Some((last_idx, last_time)) = self.last_click {
                    if last_idx == index && now.duration_since(last_time) < DOUBLE_CLICK_THRESHOLD {
                        is_double_click = true;
                    }
                }

                if is_double_click {
                    self.last_click = None;
                    if let Some(item) = self.filtered_items.get(index) {
                        let primary_action = item
                            .actions
                            .children
                            .iter()
                            .flat_map(|item| match item {
                                ActionPanelItem::Action(action) => {
                                    std::slice::from_ref(action).iter()
                                }
                                ActionPanelItem::Section(section) => section.children.iter(),
                            })
                            .next();

                        if let Some(action) = primary_action {
                            if let Some(handler) = &action.handler {
                                return Task::done(RootMessage::RunAction(handler.clone()));
                            }
                        }
                    }
                } else {
                    self.last_click = Some((index, now));
                    self.state.selected_index = index;
                    return self.scroll_to_selection();
                }

                Task::none()
            }
            RootMessage::RunAction(_) => Task::none(),
        }
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, RootMessage> {
        let mut ui_rows: Vec<Element<'_, RootMessage>> = Vec::with_capacity(30);

        let text_color = theme.colors.text;
        let secondary_text_color = iced::Color {
            a: 0.6,
            ..text_color
        };
        let selection_color = theme.colors.selection;
        let hover_color = Color::from_rgb8(39, 39, 39);

        #[cfg(feature = "soulver")]
        let has_calc = self.calculator_result.is_some();
        #[cfg(not(feature = "soulver"))]
        let has_calc = false;

        #[cfg(feature = "soulver")]
        if let Some(result) = &self.calculator_result {
            let calc_item = container(
                row![
                    text(result).color(text_color),
                    widget::space().width(Length::Fill),
                    text("Calculator").color(secondary_text_color),
                ]
                .align_y(Alignment::Center)
                .padding(12),
            )
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(selection_color)),
                ..Default::default()
            })
            .width(Length::Fill);
            ui_rows.push(calc_item.into());
        }

        let total_items = self.filtered_items.len();

        let scroll_offset = self
            .viewport
            .as_ref()
            .map(|vp| vp.absolute_offset().y)
            .unwrap_or(0.0);

        let viewport_height = self
            .viewport
            .as_ref()
            .map(|vp| vp.bounds().height)
            .unwrap_or(474.0);

        let start_index = (scroll_offset / ROW_HEIGHT).floor() as usize;
        let start_index = start_index.saturating_sub(5);

        let visible_count = (viewport_height / ROW_HEIGHT).ceil() as usize + 10;
        let end_index = (start_index + visible_count).min(total_items);

        if start_index > 0 {
            ui_rows.push(
                space()
                    .height(Length::Fixed(start_index as f32 * ROW_HEIGHT))
                    .width(Length::Fill)
                    .into(),
            );
        }

        for (i, item) in self
            .filtered_items
            .iter()
            .enumerate()
            .skip(start_index)
            .take(end_index - start_index)
        {
            let idx = i;
            let is_selected = idx == self.state.selected_index;
            let is_hovered = self.hovered_index == Some(idx);

            let background = if is_selected {
                selection_color
            } else if is_hovered {
                hover_color
            } else {
                iced::Color::TRANSPARENT
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

            let icon_element: Element<'_, RootMessage> = match &item.resolved_icon {
                ResolvedIcon::FontChar(c) => text(c)
                    .font(ICON_FONT)
                    .size(20)
                    .width(20)
                    .align_x(Alignment::Center)
                    .color(if is_selected {
                        text_color
                    } else {
                        secondary_text_color
                    })
                    .into(),
                ResolvedIcon::Svg(handle) => svg(handle.clone()).width(20).height(20).into(),
                ResolvedIcon::Image(handle) => image(handle.clone()).width(20).height(20).into(),
            };

            let mut row_content = row![icon_element]
                .align_y(Alignment::Center)
                .spacing(12)
                .height(ROW_HEIGHT);

            row_content = row_content.push(text(title).size(16).color(text_color));
            if let Some(sub) = subtitle {
                row_content = row_content.push(text(sub).size(16).color(secondary_text_color));
            }
            row_content = row_content.push(widget::space().width(Length::Fill));
            row_content = row_content.push(text(accessory).size(16).color(secondary_text_color));

            let item_container = container(row_content)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(background)),
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                })
                .center_y(ROW_HEIGHT)
                .padding([0, 8])
                .width(Length::Fill);

            let item_area = mouse_area(item_container)
                .on_press(RootMessage::ItemClicked(idx))
                .on_enter(RootMessage::ItemHovered(idx))
                .on_exit(RootMessage::ItemUnhovered(idx));

            ui_rows.push(item_area.into());
        }

        let remaining = total_items.saturating_sub(end_index);
        if remaining > 0 {
            ui_rows.push(
                space()
                    .height(Length::Fixed(remaining as f32 * ROW_HEIGHT))
                    .width(Length::Fill)
                    .into(),
            );
        }

        let content = Column::with_children(ui_rows)
            .width(Length::Fill)
            .padding([8, 8])
            .id(POSITION_TRACKER.clone());

        let (style, direction) = scrollable_style();

        scrollable(content)
            .id(self.scrollable_id.clone())
            .on_scroll(RootMessage::Scrolled)
            .height(Length::Fill)
            .style(style)
            .direction(direction)
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

        let calc_offset = if has_calc { 48.0 + 12.0 } else { 0.0 };
        let padding_top = 8.0;

        let y_offset = (base_index as f32 * ROW_HEIGHT) + padding_top + calc_offset;

        widget::operation::scroll_to(
            self.scrollable_id.clone(),
            widget::scrollable::AbsoluteOffset {
                x: 0.0,
                y: y_offset,
            },
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

fn create_action_panel(kind: &RootItemKind, id: &str) -> ActionPanel {
    let open_handler = match kind {
        RootItemKind::Extension(cmd) => {
            let cmd = cmd.clone();
            ActionHandler::new(move || Task::done(Message::LaunchCommand(cmd.clone())))
        }
        RootItemKind::App(app) => {
            let app = app.clone();
            ActionHandler::new(move || Task::done(Message::LaunchApp(app.clone())))
        }
    };

    let open_action = Action {
        title: "Open".to_string(),
        icon: None,
        handler: Some(open_handler),
    };

    let id_clone = id.to_string();
    let reset_handler =
        ActionHandler::new(move || Task::done(Message::ResetFrecency(id_clone.clone())));

    let reset_action = Action {
        title: "Reset Ranking".to_string(),
        icon: None,
        handler: Some(reset_handler),
    };

    ActionPanel {
        children: vec![
            ActionPanelItem::Action(open_action),
            ActionPanelItem::Action(reset_action),
        ],
    }
}

fn resolve_app_icon(app: &AppEntry) -> ResolvedIcon {
    let path = if std::path::Path::new(&app.icon).is_absolute() {
        Some(std::path::PathBuf::from(&app.icon))
    } else {
        freedesktop_icons::lookup(&app.icon).find()
    };

    if let Some(path) = path {
        let is_svg = path.extension().map_or(false, |e| e == "svg");
        match std::fs::read(&path) {
            Ok(bytes) => {
                if is_svg {
                    ResolvedIcon::Svg(SvgHandle::from_memory(bytes))
                } else {
                    ResolvedIcon::Image(ImageHandle::from_bytes(bytes))
                }
            }
            Err(_) => {
                if let Some(c) = crate::icons::get_icon("app-window-16") {
                    ResolvedIcon::FontChar(c.to_string())
                } else {
                    ResolvedIcon::FontChar("".to_string())
                }
            }
        }
    } else {
        if let Some(c) = crate::icons::get_icon("app-window-16") {
            ResolvedIcon::FontChar(c.to_string())
        } else {
            ResolvedIcon::FontChar("".to_string())
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

        let is_svg = path.extension().map_or(false, |e| e == "svg");
        match std::fs::read(&path) {
            Ok(bytes) => {
                if is_svg {
                    ResolvedIcon::Svg(SvgHandle::from_memory(bytes))
                } else {
                    ResolvedIcon::Image(ImageHandle::from_bytes(bytes))
                }
            }
            Err(_) => {
                if let Some(c) = crate::icons::get_icon("box-16") {
                    ResolvedIcon::FontChar(c.to_string())
                } else {
                    ResolvedIcon::FontChar("".to_string())
                }
            }
        }
    } else {
        if let Some(c) = crate::icons::get_icon("box-16") {
            ResolvedIcon::FontChar(c.to_string())
        } else {
            ResolvedIcon::FontChar("".to_string())
        }
    }
}
