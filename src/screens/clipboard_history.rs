use iced::widget::{center, text};
use iced::{Element, Task};

use crate::components::actions::ActionPanel;
use crate::screens::Shell;

pub struct ClipboardHistoryScreen;

#[derive(Clone, Debug)]
pub enum ClipboardHistoryMessage {}

impl ClipboardHistoryScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: ClipboardHistoryMessage) -> Task<ClipboardHistoryMessage> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, ClipboardHistoryMessage> {
        center(text("Clipboard History").size(24)).into()
    }
}

impl Shell for ClipboardHistoryScreen {
    fn can_search(&self) -> bool {
        true
    }

    fn on_search(&mut self, _query: &str) {}

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        None
    }
}
