use crate::types::Tree;
use iced::widget::image::Handle;
use iced::widget::scrollable::Viewport;
use reqwest::Url;

#[derive(Debug, Clone)]
pub enum Message {
    UpdateToast(String),
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    InvokeAction(String),
    ToggleActionPanel(bool),
    Scrolled(Viewport),
    ImageLoaded(String, Handle),
    LinkClicked(Url),
}
