use crate::types::Tree;
use iced::widget::image::Handle;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    ImageLoaded(String, Handle),
    InvokeAction(String),
    ToggleActionPanel(bool),
    ShowToast(String),

    Grid(crate::screens::grid::GridMessage),
    Detail(crate::screens::detail::DetailMessage),
}
