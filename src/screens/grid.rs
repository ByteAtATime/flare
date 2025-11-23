use iced::{
    Element,
    keyboard::{Key, Modifiers},
    widget::scrollable::Viewport,
};

use crate::{
    components::{
        grid::render_grid,
        types::{ActionPanel, GridProps},
    },
    globals::POSITION_TRACKER,
    screens::Shell,
};

pub struct GridScreen {
    raw_props: GridProps,
    filtered_props: GridProps,
    selected_index: usize,
    viewport: Option<Viewport>,
}

#[derive(Clone, Debug)]
pub enum GridMessage {
    KeyPressed(Key, Modifiers),
}

impl GridScreen {
    pub fn new(props: GridProps) -> Self {
        Self {
            filtered_props: props.clone(),
            raw_props: props,
            selected_index: 0,
            viewport: None,
        }
    }

    pub fn update(&mut self, message: GridMessage) {
        println!("message: {:?}", message);
    }

    pub fn view(&self) -> Element<'static, GridMessage> {
        render_grid(
            self.filtered_props.clone(),
            self.selected_index,
            POSITION_TRACKER.clone(),
            self.viewport.as_ref(),
        )
        .into()
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
        None
    }
}
