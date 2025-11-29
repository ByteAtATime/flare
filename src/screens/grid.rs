use iced::widget::scrollable::Viewport;
use iced::widget::{self, scrollable};
use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
};

use crate::{
    components::{
        actions::ActionPanel,
        grid::{GridItemContent, render_grid},
        types::GridProps,
    },
    globals::POSITION_TRACKER,
    image_cache,
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
                    let direction = match named_key {
                        Named::ArrowRight => {
                            self.state.next();
                            Some(1)
                        }
                        Named::ArrowLeft => {
                            self.state.prev();
                            Some(-1)
                        }
                        Named::ArrowUp => {
                            self.state.move_vertical(-1);
                            Some(-1)
                        }
                        Named::ArrowDown => {
                            self.state.move_vertical(1);
                            Some(1)
                        }
                        _ => None,
                    };
                    if let Some(dir) = direction {
                        return self.scroll_to_selection(dir);
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

    fn scroll_to_selection(&self, direction: i32) -> Task<GridMessage> {
        let (layout_index, header_index) = match self.state.get_layout_index(HeaderPolicy::Always) {
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

    fn on_search_text_change(&self) -> Option<&crate::components::types::CallbackInfo> {
        self.raw_props.props.on_search_text_change.as_ref()
    }

    fn load_images(&self) -> Task<crate::Message> {
        let layout_cache = crate::globals::LAYOUT_CACHE.lock().unwrap();

        let visible_range = self.viewport.as_ref().map(|vp| {
            let offset_y = vp.absolute_offset().y;
            let height = vp.bounds().height;
            (offset_y - 1500.0, offset_y + height + 1500.0)
        });

        let mut tasks = Vec::new();
        let mut col_child_idx = 0;
        let default_columns = self.filtered_props.props.columns.unwrap_or(5) as usize;

        for section in &self.filtered_props.sections {
            col_child_idx += 1;
            let columns = section
                .props
                .columns
                .map(|c| c as usize)
                .unwrap_or(default_columns);
            let start_row_idx = col_child_idx;
            let row_count = (section.items.len() + columns - 1) / columns;
            col_child_idx += row_count;

            for (chunk_idx, chunk) in section.items.chunks(columns).enumerate() {
                let current_row_idx = start_row_idx + chunk_idx;

                let is_visible = if let Some((start, end)) = visible_range {
                    if let Some(bounds) = layout_cache.get(&current_row_idx) {
                        let row_top = bounds.y;
                        let row_bottom = bounds.y + bounds.height;
                        row_bottom >= start && row_top <= end
                    } else {
                        current_row_idx < 10
                    }
                } else {
                    current_row_idx < 10
                };

                if is_visible {
                    for item in chunk {
                        if let Some(GridItemContent::Image(url)) = &item.props.content {
                            if url.starts_with("http") {
                                tasks.push(image_cache::fetch(url.clone()));
                            }
                        }
                    }
                }
            }
        }

        Task::batch(tasks)
    }
}
