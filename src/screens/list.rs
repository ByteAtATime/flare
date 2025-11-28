use iced::widget::scrollable::Viewport;
use iced::widget::{self, markdown};
use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
};

use crate::{
    components::{actions::ActionPanel, list::render_list, types::ListProps},
    globals::POSITION_TRACKER,
    screens::Shell,
    selection::{HeaderPolicy, Section, SelectionState, scroll_to},
    utils::open_url,
};

pub struct ListScreen {
    raw_props: ListProps,
    filtered_props: ListProps,
    state: SelectionState<crate::components::types::ListItemProps>,
    viewport: Option<Viewport>,
    pub scrollable_id: widget::Id,
    detail_cache: Option<Vec<markdown::Item>>,
}

#[derive(Clone, Debug)]
pub enum ListMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
    Detail(crate::screens::detail::DetailMessage),
}

impl ListScreen {
    pub fn new(
        props: ListProps,
        viewport: Option<Viewport>,
        scrollable_id: Option<widget::Id>,
    ) -> Self {
        let state = Self::create_state(&props);

        let mut screen = Self {
            filtered_props: props.clone(),
            raw_props: props,
            state,
            viewport,
            scrollable_id: scrollable_id.unwrap_or_else(widget::Id::unique),
            detail_cache: None,
        };
        screen.update_detail_cache();
        screen
    }

    fn create_state(props: &ListProps) -> SelectionState<crate::components::types::ListItemProps> {
        let sections = props
            .sections
            .iter()
            .map(|s| Section {
                title: s.props.title.clone(),
                items: s.items.clone(),
                columns: Some(1),
            })
            .collect();
        SelectionState::new(sections, 1)
    }

    fn update_detail_cache(&mut self) {
        if self.filtered_props.props.is_showing_detail {
            if let Some(item) = self.state.selected_item() {
                if let Some(detail) = &item.props.detail {
                    self.detail_cache = Some(markdown::parse(&detail.props.markdown).collect());
                    return;
                }
            }
        }
        self.detail_cache = None;
    }

    pub fn update(&mut self, message: ListMessage) -> Task<ListMessage> {
        match message {
            ListMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    use iced::keyboard::key::Named;
                    let moved = match named_key {
                        Named::ArrowDown => {
                            self.state.move_vertical(1);
                            self.update_detail_cache();
                            true
                        }
                        Named::ArrowUp => {
                            self.state.move_vertical(-1);
                            self.update_detail_cache();
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
            ListMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
            ListMessage::Detail(msg) => match msg {
                crate::screens::detail::DetailMessage::LinkClicked(url) => {
                    let _ = open_url(&url);
                    Task::none()
                }
                _ => Task::none(),
            },
        }
    }

    pub fn get_viewport(&self) -> Option<Viewport> {
        self.viewport.clone()
    }

    pub fn view(&self) -> Element<'_, ListMessage> {
        render_list(
            &self.filtered_props,
            self.state.selected_index,
            POSITION_TRACKER.clone(),
            self.scrollable_id.clone(),
            self.detail_cache.as_ref(),
        )
    }

    fn scroll_to_selection(&self) -> Task<ListMessage> {
        let container_index = match self.state.get_layout_index(HeaderPolicy::IfTitleNotEmpty) {
            Some(idx) => idx,
            None => return Task::none(),
        };

        scroll_to(
            self.scrollable_id.clone(),
            self.viewport.as_ref(),
            container_index,
        )
    }
}

impl Shell for ListScreen {
    fn can_search(&self) -> bool {
        true
    }

    fn on_search(&mut self, query: &str) {
        let query_lower = query.to_lowercase();

        if query.is_empty() {
            self.filtered_props = self.raw_props.clone();
        } else if self.raw_props.props.on_search_text_change.is_some() {
            self.filtered_props = self.raw_props.clone();
        } else {
            let mut new_props = self.raw_props.clone();
            new_props.sections.retain_mut(|section| {
                section.items.retain(|item| {
                    item.props.title.to_lowercase().contains(&query_lower)
                        || item
                            .props
                            .subtitle
                            .as_ref()
                            .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                });
                !section.items.is_empty()
            });
            self.filtered_props = new_props;
        }

        self.state = Self::create_state(&self.filtered_props);
        self.update_detail_cache();
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        self.state
            .selected_item_mut()
            .and_then(|item| item.props.actions.as_mut())
    }

    fn get_search_bar_accessory(&self) -> Option<&crate::components::dropdown::Dropdown> {
        self.raw_props.props.search_bar_accessory.as_ref()
    }

    fn set_dropdown_value(&mut self, value: &str) {
        if let Some(dropdown) = self.raw_props.props.search_bar_accessory.as_mut() {
            dropdown.props.value = Some(value.to_string());
        }
    }

    fn on_search_text_change(&self) -> Option<&crate::components::types::CallbackInfo> {
        self.raw_props.props.on_search_text_change.as_ref()
    }
}
