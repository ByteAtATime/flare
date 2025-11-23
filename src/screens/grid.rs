use iced::widget::scrollable::Viewport;
use iced::widget::{self, scrollable};
use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
    widget::operation,
};

use crate::{
    components::{
        grid::render_grid,
        types::{ActionPanel, GridProps},
    },
    globals::{LAYOUT_CACHE, POSITION_TRACKER},
    screens::Shell,
};

pub struct GridScreen {
    raw_props: GridProps,
    filtered_props: GridProps,
    selected_index: usize,
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
        Self {
            filtered_props: props.clone(),
            raw_props: props,
            selected_index: 0,
            viewport,
            scrollable_id: scrollable_id.unwrap_or_else(widget::Id::unique),
        }
    }

    pub fn update(&mut self, message: GridMessage) -> Task<GridMessage> {
        match message {
            GridMessage::KeyPressed(key, _modifiers) => {
                if let Key::Named(named_key) = key {
                    use iced::keyboard::key::Named;
                    let moved = match named_key {
                        Named::ArrowRight => {
                            self.select_next();
                            true
                        }
                        Named::ArrowLeft => {
                            self.select_prev();
                            true
                        }
                        Named::ArrowUp => {
                            self.select_up();
                            true
                        }
                        Named::ArrowDown => {
                            self.select_down();
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
            self.selected_index,
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
        let container_index = match self.get_selection_container_index() {
            Some(idx) => idx,
            None => return Task::none(),
        };

        let target_bounds = match LAYOUT_CACHE
            .lock()
            .ok()
            .and_then(|cache| cache.get(&container_index).copied())
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
        }
    }

    fn select_prev(&mut self) {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + total - 1) % total;
        }
    }

    fn select_up(&mut self) {
        self.move_vertical(-1);
    }

    fn select_down(&mut self) {
        self.move_vertical(1);
    }

    fn move_vertical(&mut self, direction: i32) {
        let default_cols = self.filtered_props.columns.unwrap_or(5).max(1) as usize;

        struct Section {
            start: usize,
            count: usize,
            cols: usize,
            rows: usize,
        }

        let (sections, current_pos): (Vec<_>, Option<(usize, usize, usize)>) = self
            .filtered_props
            .sections
            .iter()
            .enumerate()
            .scan(0, |acc, (i, section)| {
                let start = *acc;
                let count = section.items.len();
                let cols = section
                    .columns
                    .map(|c| c as usize)
                    .unwrap_or(default_cols)
                    .max(1);
                let rows = if count > 0 {
                    (count + cols - 1) / cols
                } else {
                    0
                };

                let pos = if self.selected_index >= start && self.selected_index < start + count {
                    let local = self.selected_index - start;
                    Some((i, local / cols, local % cols))
                } else {
                    None
                };

                *acc += count;
                Some((
                    Section {
                        start,
                        count,
                        cols,
                        rows,
                    },
                    pos,
                ))
            })
            .fold(
                (Vec::new(), None),
                |(mut secs, mut pos), (sec, curr_pos)| {
                    secs.push(sec);
                    if pos.is_none() && curr_pos.is_some() {
                        pos = curr_pos;
                    }
                    (secs, pos)
                },
            );

        let (mut sec_idx, current_row, current_col) = match current_pos {
            Some(pos) => pos,
            None => {
                if sections.iter().any(|s| s.count > 0) {
                    self.selected_index = 0;
                }
                return;
            }
        };

        let mut search_row = current_row as i32 + direction;

        for _ in 0..(sections.len() * 2) {
            if sec_idx >= sections.len() {
                sec_idx = 0;
                search_row = 0;
            }

            let sec = &sections[sec_idx];

            if sec.count == 0 {
                let offset = if direction > 0 { 1 } else { sections.len() - 1 };
                sec_idx = (sec_idx + offset) % sections.len();
                search_row = if direction > 0 { 0 } else { i32::MAX };
                continue;
            }

            if search_row == i32::MAX {
                search_row = sec.rows.saturating_sub(1) as i32;
            }

            if search_row < 0 {
                sec_idx = (sec_idx + sections.len() - 1) % sections.len();
                search_row = i32::MAX;
                continue;
            }

            if search_row as usize >= sec.rows {
                sec_idx = (sec_idx + 1) % sections.len();
                search_row = 0;
                continue;
            }

            let row_start = search_row as usize * sec.cols;
            let items_in_row = (sec.count - row_start).min(sec.cols);
            self.selected_index = sec.start + row_start + current_col.min(items_in_row - 1);
            return;
        }
    }

    pub fn get_selection_container_index(&self) -> Option<usize> {
        let default_columns = self.filtered_props.columns.unwrap_or(5).max(1) as usize;
        let mut position_index = 0;
        let mut item_cursor = 0;

        for section in &self.filtered_props.sections {
            position_index += 1;

            let section_len = section.items.len();
            if self.selected_index >= item_cursor && self.selected_index < item_cursor + section_len
            {
                let cols = section
                    .columns
                    .map(|c| c as usize)
                    .unwrap_or(default_columns)
                    .max(1);
                let local_index = self.selected_index - item_cursor;
                let row_offset = local_index / cols;
                return Some(position_index + row_offset);
            }

            item_cursor += section_len;
            let cols = section
                .columns
                .map(|c| c as usize)
                .unwrap_or(default_columns)
                .max(1);
            let rows = (section_len + cols - 1) / cols;
            position_index += rows;
        }

        None
    }

    fn count_total_items(&self) -> usize {
        self.filtered_props
            .sections
            .iter()
            .map(|s| s.items.len())
            .sum()
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
        } else if self.raw_props.on_search_text_change.is_some() {
            self.filtered_props = self.raw_props.clone();
        } else {
            let mut new_props = self.raw_props.clone();
            new_props.sections.retain_mut(|section| {
                section.items.retain(|item| {
                    item.title.to_lowercase().contains(&query_lower)
                        || item
                            .subtitle
                            .as_ref()
                            .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                });
                !section.items.is_empty()
            });
            self.filtered_props = new_props;
        }

        self.selected_index = 0;
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
                    .and_then(|item| item.actions.as_mut());
            }
            global_index += section_len;
        }
        None
    }
}
