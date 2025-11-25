use iced::widget::scrollable::Viewport;
use iced::widget::{self, markdown, scrollable};
use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
    widget::operation,
};

use crate::{
    components::{actions::ActionPanel, list::render_list, types::ListProps},
    globals::{LAYOUT_CACHE, POSITION_TRACKER},
    screens::Shell,
    utils::open_url,
};

pub struct ListScreen {
    raw_props: ListProps,
    filtered_props: ListProps,
    selected_index: usize,
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
        let mut screen = Self {
            filtered_props: props.clone(),
            raw_props: props,
            selected_index: 0,
            viewport,
            scrollable_id: scrollable_id.unwrap_or_else(widget::Id::unique),
            detail_cache: None,
        };
        screen.update_detail_cache();
        screen
    }

    fn update_detail_cache(&mut self) {
        if self.filtered_props.props.is_showing_detail {
            let mut cursor = 0;
            for section in &self.filtered_props.sections {
                if self.selected_index >= cursor
                    && self.selected_index < cursor + section.items.len()
                {
                    if let Some(item) = section.items.get(self.selected_index - cursor) {
                        if let Some(detail) = &item.props.detail {
                            self.detail_cache =
                                Some(markdown::parse(&detail.props.markdown).collect());
                            return;
                        }
                    }
                }
                cursor += section.items.len();
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
        let content = render_list(
            &self.filtered_props,
            self.selected_index,
            POSITION_TRACKER.clone(),
            self.detail_cache.as_ref(),
        );

        scrollable(content)
            .id(self.scrollable_id.clone())
            .on_scroll(ListMessage::Scrolled)
            .height(iced::Length::Fill)
            .into()
    }

    fn scroll_to_selection(&self) -> Task<ListMessage> {
        let target_bounds = match LAYOUT_CACHE
            .lock()
            .ok()
            .and_then(|cache| cache.get(&self.selected_index).copied())
        {
            Some(bounds) => bounds,
            None => return Task::none(),
        };

        let offset = match &self.viewport {
            Some(vp) => {
                let view_top = vp.absolute_offset().y;
                let view_bottom = view_top + vp.bounds().height;
                let target_top = target_bounds.y;
                let target_bottom = target_top + target_bounds.height;

                if target_top < view_top {
                    Some(target_top)
                } else if target_bottom > view_bottom {
                    Some(target_bottom - vp.bounds().height)
                } else {
                    None
                }
            }
            None => Some(target_bounds.y),
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
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
            self.update_detail_cache();
        }
    }

    fn select_prev(&mut self) {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + total - 1) % total;
            self.update_detail_cache();
        }
    }

    fn count_total_items(&self) -> usize {
        self.filtered_props
            .sections
            .iter()
            .map(|s| s.items.len())
            .sum()
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

        self.selected_index = 0;
        self.update_detail_cache();
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        let mut global_index = 0;
        for section in &mut self.filtered_props.sections {
            let section_len = section.items.len();
            if self.selected_index >= global_index
                && self.selected_index < global_index + section_len
            {
                let item_index = self.selected_index - global_index;
                return section
                    .items
                    .get_mut(item_index)
                    .and_then(|item| item.props.actions.as_mut());
            }
            global_index += section_len;
        }
        None
    }
}
