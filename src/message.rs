use crate::apps::AppEntry;
use crate::components::actions::ActionHandler;
use crate::extensions::ExtensionCommand;
use crate::types::Tree;
use iced::widget::image::Handle;
use iced::window;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    EscapePressed,
    ImageLoaded(String, Handle),
    InvokeAction(ActionHandler),
    ToggleActionPanel(bool),
    ActionPanelSearchChanged(String),
    ActionPanelSelect(usize),
    ActionPanelMoveUp,
    ActionPanelMoveDown,
    ShowToast(String),
    DropdownChanged(String),
    LaunchCommand(ExtensionCommand),
    LaunchApp(AppEntry),
    ResetFrecency(String),
    PopToRoot,

    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ToggleWindow,
    OpenSettings,
    SettingsWindowOpened(window::Id),

    Settings(crate::screens::settings::SettingsMessage),
    OpenUrl(String),

    Tick(Instant),

    Root(crate::screens::root::RootMessage),
    Grid(crate::screens::grid::GridMessage),
    Detail(crate::screens::detail::DetailMessage),
    List(crate::screens::list::ListMessage),
}
