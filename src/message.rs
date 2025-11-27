use crate::extensions::ExtensionCommand;
use crate::types::Tree;
use iced::widget::image::Handle;
use iced::window;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    ImageLoaded(String, Handle),
    InvokeAction(String),
    ToggleActionPanel(bool),
    ShowToast(String),
    DropdownChanged(String),
    LaunchCommand(ExtensionCommand),
    PopToRoot,

    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ToggleWindow,
    OpenSettings,
    SettingsWindowOpened(window::Id),

    Settings(crate::screens::settings::SettingsMessage),
    OpenUrl(String),

    Root(crate::screens::root::RootMessage),
    Grid(crate::screens::grid::GridMessage),
    Detail(crate::screens::detail::DetailMessage),
    List(crate::screens::list::ListMessage),
}
