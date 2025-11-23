use crate::types::Tree;
use iced::widget::image::Handle;
use iced::widget::scrollable::Viewport;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    ImageLoaded(String, Handle),
    InvokeAction(String),
    ToggleActionPanel(bool),
    Scrolled(Viewport),
    LinkClicked(String),
    ShowToast(String),

    Grid(crate::screens::grid::GridMessage),
    Detail(crate::screens::detail::DetailMessage),
}
