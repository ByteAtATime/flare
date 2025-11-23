use iced::{Element, widget::text};

use crate::{
    components::{
        grid::render_grid,
        types::{ActionPanel, GridProps},
    },
    globals::POSITION_TRACKER,
    screens::Shell,
};

pub struct GridScreen {
    props: GridProps,
}

#[derive(Clone, Debug)]
pub enum GridMessage {}

impl GridScreen {
    pub fn new(props: GridProps) -> Self {
        Self { props }
    }

    pub fn update(&mut self, message: GridMessage) {
        println!("message: {:?}", message);
    }

    pub fn view(&self) -> Element<'static, GridMessage> {
        println!("props: {:?}", self.props);
        render_grid(self.props.clone(), 0, POSITION_TRACKER.clone(), None).into()
    }
}

impl Shell for GridScreen {
    fn can_search(&self) -> bool {
        true
    }

    fn on_search(&mut self, query: &str) {
        println!("on search: {}", query);
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        None
    }
}
