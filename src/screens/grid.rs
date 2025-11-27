use iced::widget::scrollable::Viewport;
use iced::widget::{self, scrollable};
use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
};

use crate::{
    components::{actions::ActionPanel, grid::render_grid, types::GridProps},
    globals::POSITION_TRACKER,
    screens::Shell,
    selection::{HeaderPolicy, Section, SelectionState, scroll_to},
};

pub struct GridScreen {
    raw_props: GridProps,
    filtered_props: GridProps,
    state: SelectionState<crate::components::types::GridItemProps>,
    viewport: Option<Viewport>,
    pub scrollable_id: widget::Id,
}

#[derive(Clone, Debug)]
pub enum GridMessage {
    KeyPressed(Key, Modifiers),
    Scrolled(Viewport),
}

impl GridScreen {
    pub fn new(
        props: GridProps,
        viewport: Option<Viewport>,
        scrollable_id: Option<widget::Id>,
    ) -> Self {
        let state = Self::create_state(&props);

        Self {
            filtered_props: props.clone(),
            raw_props: props,
            state,
            viewport,
            scrollable_id: scrollable_id.unwrap_or_else(widget::Id::unique),
        }
    }

    fn create_state(props: &GridProps) -> SelectionState<crate::components::types::GridItemProps> {
        let sections = props
            .sections
            .iter()
            .map(|s| Section {
                title: s.props.title.clone(),
                items: s.items.clone(),
                columns: s.props.columns,
            })
            .collect();
        SelectionState::new(sections, props.props.columns.unwrap_or(5).max(1) as usize)
    }

    pub fn update(&mut self, message: GridMessage) -> Task<GridMessage> {
        match message {
            GridMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    use iced::keyboard::key::Named;
                    let moved = match named_key {
                        Named::ArrowRight => {
                            self.state.next();
                            true
                        }
                        Named::ArrowLeft => {
                            self.state.prev();
                            true
                        }
                        Named::ArrowUp => {
                            self.state.move_vertical(-1);
                            true
                        }
                        Named::ArrowDown => {
                            self.state.move_vertical(1);
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
            GridMessage::Scrolled(viewport) => {
                self.viewport = Some(viewport);
                Task::none()
            }
        }
    }

    pub fn get_viewport(&self) -> Option<Viewport> {
        self.viewport.clone()
    }

    pub fn view(&self) -> Element<'static, GridMessage> {
        let content = render_grid(
            &self.filtered_props,
            self.state.selected_index,
            POSITION_TRACKER.clone(),
            self.viewport.as_ref(),
        );

        scrollable(content)
            .id(self.scrollable_id.clone())
            .on_scroll(GridMessage::Scrolled)
            .height(iced::Length::Fill)
            .into()
    }

    fn scroll_to_selection(&self) -> Task<GridMessage> {
        let container_index = match self.state.get_layout_index(HeaderPolicy::Always) {
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

impl Shell for GridScreen {
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
}
