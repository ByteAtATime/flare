use crate::components::{self, Component};
use crate::types::Tree;
use iced::widget::markdown;
use iced::widget::scrollable::Viewport;

#[derive(Default)]
pub struct State {
    pub toast_message: String,
    pub search_text: String,
    pub tree: Option<Tree>,
    pub filtered_tree: Option<Tree>,
    pub selected_index: usize,
    pub selected_actions: Vec<components::actions::ActionPanelItem>,
    pub action_panel_visible: bool,
    pub viewport: Option<Viewport>,
}

impl State {
    pub fn update_tree(&mut self, mut tree: Tree) {
        for component in &mut tree.children {
            if let Component::Detail(props) = component {
                let items: Vec<_> = markdown::parse(&props.props.markdown).collect();
                props.parsed = Some(items);
            }
        }

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
                Component::Grid(p) => p
                    .props
                    .on_search_text_change
                    .as_ref()
                    .map(|cb| cb.id.clone()),
                _ => None,
            });

        self.update_filtered();
        self.selected_index = 0;
        self.update_selected_actions();

        callback
    }

    fn update_filtered(&mut self) {
        if let Some(raw_tree) = &self.tree {
            let query = self.search_text.to_lowercase();

            if query.is_empty() {
                self.filtered_tree = Some(raw_tree.clone());
                return;
            }

            let mut new_tree = raw_tree.clone();
            new_tree.children =
                new_tree
                    .children
                    .iter()
                    .map(|component| match component {
                        Component::Grid(props) => {
                            if props.props.on_search_text_change.is_some() {
                                Component::Grid(props.clone())
                            } else {
                                let mut new_props = props.clone();
                                new_props.sections.retain_mut(|section| {
                                    section.items.retain(|item| {
                                        item.props.title.to_lowercase().contains(&query)
                                            || item.props.subtitle.as_ref().map_or(false, |s| {
                                                s.to_lowercase().contains(&query)
                                            })
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
            .and_then(|item| item.props.actions.as_ref())
            .map(|p| p.children.clone())
            .unwrap_or_default();
    }
}
