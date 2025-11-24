use iced::{
    Element, Task,
    keyboard::{Key, Modifiers},
    widget::markdown,
};

use crate::screens::Shell;
use crate::{
    components::{actions::ActionPanel, detail::render_detail, types::DetailProps},
    utils::open_url,
};

pub struct DetailScreen {
    props: DetailProps,
    parsed: Vec<markdown::Item>,
}

#[derive(Clone, Debug)]
pub enum DetailMessage {
    LinkClicked(String),
    KeyPressed(Key, Modifiers),
}

impl DetailScreen {
    pub fn new(props: DetailProps) -> Self {
        let items: Vec<_> = markdown::parse(&props.props.markdown).collect();
        let parsed = items;
        Self { props, parsed }
    }

    pub fn view(&self) -> Element<'_, DetailMessage> {
        render_detail(&self.props, &self.parsed).into()
    }

    pub fn update(&mut self, message: DetailMessage) -> Task<DetailMessage> {
        match message {
            DetailMessage::LinkClicked(url) => {
                let _ = open_url(&url);
            }
            _ => {}
        }
        Task::none()
    }
}

impl Shell for DetailScreen {
    fn can_search(&self) -> bool {
        false
    }

    fn on_search(&mut self, _query: &str) {}

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        self.props.props.actions.as_mut()
    }
}
