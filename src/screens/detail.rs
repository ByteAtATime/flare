use iced::{
    Element, Length, Task,
    keyboard::{Key, Modifiers},
    widget::{container, markdown},
};

use crate::components::{
    detail::render_detail,
    types::{ActionPanel, DetailProps},
};
use crate::screens::Shell;

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
        let items: Vec<_> = markdown::parse(&props.markdown).collect();
        let parsed = items;
        Self { props, parsed }
    }

    pub fn view(&self) -> Element<'_, DetailMessage> {
        container(render_detail(&self.props, &self.parsed))
            .padding(20)
            .width(Length::Fill)
            .into()
    }

    pub fn update(&mut self, _message: DetailMessage) -> Task<DetailMessage> {
        Task::none()
    }
}

impl Shell for DetailScreen {
    fn can_search(&self) -> bool {
        false
    }

    fn on_search(&mut self, _query: &str) {}

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        self.props.actions.as_mut()
    }
}
