use crate::types::Tree;
use iced::widget::scrollable::Viewport;

#[derive(Debug, Clone)]
pub enum Message {
    UpdateToast(String),
    UpdateTree(Tree),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    InvokeAction(String),
    ToggleActionPanel(bool),
    Scrolled(Viewport),
    ScrollCompleted,
}
