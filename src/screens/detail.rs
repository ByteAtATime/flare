use iced::{Element, Length, Task, widget::container, widget::markdown};

use crate::components::{
    detail::render_detail,
    types::{ActionPanel, DetailProps},
};
use crate::message::Message;
use crate::screens::Shell;

pub struct DetailScreen {
    props: DetailProps,
    parsed: Vec<markdown::Item>,
}

impl DetailScreen {
    pub fn new(props: DetailProps) -> Self {
        let items: Vec<_> = markdown::parse(&props.markdown).collect();
        let parsed = items;
        Self { props, parsed }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(render_detail(&self.props, &self.parsed))
            .padding(20)
            .width(Length::Fill)
            .into()
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
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
