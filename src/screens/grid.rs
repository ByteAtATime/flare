use iced::{Element, widget::text};

use crate::{
    components::types::{ActionPanel, GridProps},
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
        text!("grid").into()
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
