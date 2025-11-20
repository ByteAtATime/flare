use crate::components::{self, Component};
use crate::types::Tree;
use iced::widget::scrollable::Viewport;

#[derive(Default)]
pub struct State {
    pub toast_message: String,
    pub search_text: String,
    pub tree: Option<Tree>,
    pub filtered_tree: Option<Tree>,
    pub selected_index: usize,
    pub selected_actions: Vec<components::types::Action>,
    pub action_panel_visible: bool,
    pub viewport: Option<Viewport>,
}

impl State {
    pub fn update_tree(&mut self, tree: Tree) {
        self.tree = Some(tree);
        self.update_filtered();

        let total = self.count_total_items();
        if self.selected_index >= total && total > 0 {
            self.selected_index = total - 1;
        }
        self.update_selected_actions();
    }

    pub fn update_search(&mut self, text: String) -> Option<String> {
        self.search_text = text;

        let callback = self
            .tree
            .as_ref()
            .and_then(|t| t.children.first())
            .and_then(|c| match c {
                Component::Grid(p) => p.on_search_text_change.as_ref().map(|cb| cb.id.clone()),
                _ => None,
            });

        self.update_filtered();
        self.selected_index = 0;
        self.update_selected_actions();

        callback
    }

    pub fn select_next(&mut self) -> bool {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
            self.update_selected_actions();
            true
        } else {
            false
        }
    }

    pub fn select_prev(&mut self) -> bool {
        let total = self.count_total_items();
        if total > 0 {
            self.selected_index = if self.selected_index == 0 {
                total - 1
            } else {
                self.selected_index - 1
            };
            self.update_selected_actions();
            true
        } else {
            false
        }
    }

    pub fn select_up(&mut self) -> bool {
        self.move_vertical(-1)
    }

    pub fn select_down(&mut self) -> bool {
        self.move_vertical(1)
    }

    fn move_vertical(&mut self, direction: i32) -> bool {
        let grid_props = match self.filtered_tree.as_ref().and_then(|t| t.children.first()) {
            Some(Component::Grid(p)) => p,
            _ => return false,
        };

        let default_columns = grid_props.columns.unwrap_or(5) as usize;

        struct Sec {
            start: usize,
            count: usize,
            cols: usize,
            rows: usize,
        }

        let mut sections = Vec::with_capacity(grid_props.sections.len());
        let mut acc_idx = 0;
        let mut curr_pos = None;

        for (i, section) in grid_props.sections.iter().enumerate() {
            let count = section.items.len();
            let cols = section
                .columns
                .map(|c| c as usize)
                .unwrap_or(default_columns);
            let cols = if cols == 0 { 1 } else { cols };
            let rows = if count > 0 {
                (count + cols - 1) / cols
            } else {
                0
            };

            if self.selected_index >= acc_idx && self.selected_index < acc_idx + count {
                let local = self.selected_index - acc_idx;
                curr_pos = Some((i, local / cols, local % cols));
            }

            sections.push(Sec {
                start: acc_idx,
                count,
                cols,
                rows,
            });
            acc_idx += count;
        }

        let (start_sec, start_row, start_col) = match curr_pos {
            Some(p) => p,
            None => {
                if sections.iter().any(|s| s.count > 0) {
                    self.selected_index = 0;
                    self.update_selected_actions();
                    return true;
                }
                return false;
            }
        };

        let mut search_sec_idx = start_sec as i32;
        let mut search_row_idx = (start_row as i32) + direction;

        loop {
            if search_sec_idx < 0 || search_sec_idx >= sections.len() as i32 {
                return false;
            }

            let sec = &sections[search_sec_idx as usize];

            if sec.count == 0 {
                search_sec_idx += direction;
                search_row_idx = if direction > 0 { 0 } else { -1 };
                continue;
            }

            if search_row_idx == -1 {
                search_row_idx = (sec.rows as i32) - 1;
            }

            if search_row_idx >= 0 && search_row_idx < sec.rows as i32 {
                let target_col = if start_col >= sec.cols {
                    sec.cols - 1
                } else {
                    start_col
                };

                let mut target_local = (search_row_idx as usize) * sec.cols + target_col;
                if target_local >= sec.count {
                    target_local = sec.count - 1;
                }

                self.selected_index = sec.start + target_local;
                self.update_selected_actions();
                return true;
            }

            search_sec_idx += direction;
            if direction > 0 {
                search_row_idx = 0;
            } else {
                search_row_idx = -1;
            }
        }
    }

    pub fn get_selection_container_index(&self) -> Option<usize> {
        let grid_props = self
            .filtered_tree
            .as_ref()?
            .children
            .first()
            .and_then(|c| match c {
                Component::Grid(props) => Some(props),
                _ => None,
            })?;

        let mut position_index = 0;
        let mut item_cursor = 0;

        for section in &grid_props.sections {
            position_index += 1; // title

            let section_len = section.items.len();
            if self.selected_index >= item_cursor && self.selected_index < item_cursor + section_len
            {
                let columns = section.columns.or(grid_props.columns).unwrap_or(5) as usize;
                let columns = if columns == 0 { 1 } else { columns };

                let local_index = self.selected_index - item_cursor;
                let row_offset = local_index / columns;

                return Some(position_index + row_offset);
            }

            item_cursor += section_len;

            let columns = section.columns.or(grid_props.columns).unwrap_or(5) as usize;
            let columns = if columns == 0 { 1 } else { columns };
            let rows = (section_len + columns - 1) / columns;
            position_index += rows;
        }
        None
    }

    fn update_filtered(&mut self) {
        if let Some(raw_tree) = &self.tree {
            let query = self.search_text.to_lowercase();

            if query.is_empty() {
                self.filtered_tree = Some(raw_tree.clone());
                return;
            }

            let mut new_tree = raw_tree.clone();
            new_tree.children = new_tree
                .children
                .iter()
                .map(|component| match component {
                    Component::Grid(props) => {
                        if props.on_search_text_change.is_some() {
                            Component::Grid(props.clone())
                        } else {
                            let mut new_props = props.clone();
                            new_props.sections.retain_mut(|section| {
                                section.items.retain(|item| {
                                    item.title.to_lowercase().contains(&query)
                                        || item
                                            .subtitle
                                            .as_ref()
                                            .map_or(false, |s| s.to_lowercase().contains(&query))
                                });
                                !section.items.is_empty()
                            });
                            Component::Grid(new_props)
                        }
                    }
                    _ => component.clone(),
                })
                .collect();

            self.filtered_tree = Some(new_tree);
        } else {
            self.filtered_tree = None;
        }
    }

    pub fn count_total_items(&self) -> usize {
        self.filtered_tree
            .as_ref()
            .and_then(|t| t.children.first())
            .and_then(|c| match c {
                Component::Grid(p) => Some(p.sections.iter().map(|s| s.items.len()).sum()),
                _ => None,
            })
            .unwrap_or(0)
    }

    fn update_selected_actions(&mut self) {
        self.selected_actions = self
            .filtered_tree
            .as_ref()
            .and_then(|tree| tree.children.first())
            .and_then(|component| {
                if let Component::Grid(grid_props) = component {
                    let mut global_index = 0;
                    for section in &grid_props.sections {
                        let section_len = section.items.len();
                        if self.selected_index < global_index + section_len {
                            return section.items.get(self.selected_index - global_index);
                        }
                        global_index += section_len;
                    }
                }
                None
            })
            .and_then(|item| item.actions.as_ref())
            .map(|p| p.children.clone())
            .unwrap_or_default();
    }
}
