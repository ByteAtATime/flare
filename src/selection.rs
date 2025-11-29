use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section<T> {
    pub title: String,
    pub items: Vec<T>,
    pub columns: Option<i32>,
}

pub struct SelectionState<T> {
    pub sections: Vec<Section<T>>,
    pub selected_index: usize,
    pub default_columns: usize,
}

pub enum HeaderPolicy {
    Always,
    IfTitleNotEmpty,
    Never,
}

impl<T> SelectionState<T> {
    pub fn new(sections: Vec<Section<T>>, default_columns: usize) -> Self {
        Self {
            sections,
            selected_index: 0,
            default_columns: std::cmp::max(1, default_columns),
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        let mut cursor = 0;
        for section in &self.sections {
            if self.selected_index >= cursor && self.selected_index < cursor + section.items.len() {
                return section.items.get(self.selected_index - cursor);
            }
            cursor += section.items.len();
        }
        None
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut T> {
        let mut cursor = 0;
        for section in &mut self.sections {
            if self.selected_index >= cursor && self.selected_index < cursor + section.items.len() {
                return section.items.get_mut(self.selected_index - cursor);
            }
            cursor += section.items.len();
        }
        None
    }

    pub fn next(&mut self) {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
        }
    }

    pub fn prev(&mut self) {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + total - 1) % total;
        }
    }

    pub fn move_vertical(&mut self, direction: i32) {
        struct SectionInfo {
            start: usize,
            count: usize,
            cols: usize,
            rows: usize,
        }

        let default_cols = self.default_columns;

        let (sections, current_pos): (Vec<_>, Option<(usize, usize, usize)>) = self
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
                    SectionInfo {
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

    pub fn get_layout_index(&self, header_policy: HeaderPolicy) -> Option<(usize, Option<usize>)> {
        let mut position_index = 0;
        let mut item_cursor = 0;

        for section in &self.sections {
            let header_index = position_index;
            let has_header = match header_policy {
                HeaderPolicy::Always => {
                    position_index += 1;
                    true
                }
                HeaderPolicy::IfTitleNotEmpty => {
                    if !section.title.is_empty() {
                        position_index += 1;
                        true
                    } else {
                        false
                    }
                }
                HeaderPolicy::Never => false,
            };

            let section_len = section.items.len();
            if self.selected_index >= item_cursor && self.selected_index < item_cursor + section_len
            {
                let cols = section
                    .columns
                    .map(|c| c as usize)
                    .unwrap_or(self.default_columns)
                    .max(1);
                let local_index = self.selected_index - item_cursor;
                let row_offset = local_index / cols;
                let row_index = position_index + row_offset;
                let header = if row_offset == 0 && has_header {
                    Some(header_index)
                } else {
                    None
                };
                return Some((row_index, header));
            }

            item_cursor += section_len;
            let cols = section
                .columns
                .map(|c| c as usize)
                .unwrap_or(self.default_columns)
                .max(1);
            let rows = (section_len + cols - 1) / cols;
            position_index += rows;
        }

        None
    }

    fn count_total_items(&self) -> usize {
        self.sections.iter().map(|s| s.items.len()).sum()
    }
}

pub fn scroll_to<Message: 'static>(
    id: iced::widget::Id,
    viewport: Option<&iced::widget::scrollable::Viewport>,
    layout_index: usize,
    header_index: Option<usize>,
    direction: i32,
) -> iced::Task<Message> {
    use crate::globals::LAYOUT_CACHE;
    use iced::widget::operation;
    use iced::widget::scrollable;

    let cache = match LAYOUT_CACHE.lock().ok() {
        Some(c) => c,
        None => return iced::Task::none(),
    };

    let target_bounds = match cache.get(&layout_index).copied() {
        Some(b) => b,
        None => return iced::Task::none(),
    };

    let header_bounds = header_index.and_then(|idx| cache.get(&idx).copied());

    drop(cache);

    let offset = match viewport {
        Some(vp) => {
            let view_top = vp.absolute_offset().y;
            let view_bottom = view_top + vp.bounds().height;
            let target_top = target_bounds.y;
            let target_bottom = target_top + target_bounds.height;

            if target_top < view_top {
                if direction < 0 {
                    if let Some(hb) = header_bounds {
                        Some(hb.y)
                    } else {
                        Some(target_top)
                    }
                } else {
                    Some(target_top)
                }
            } else if target_bottom > view_bottom {
                Some(target_bottom - vp.bounds().height)
            } else {
                None
            }
        }
        None => Some(target_bounds.y),
    };

    match offset {
        Some(y) => operation::scroll_to(id, scrollable::AbsoluteOffset { x: 0.0, y }),
        None => iced::Task::none(),
    }
}
