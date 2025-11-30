use crate::apps::AppEntry;
use crate::components::actions::ActionHandler;
use crate::extensions::ExtensionCommand;
use crate::transport::{SidecarReader, SidecarWriter};
use crate::types::{SidecarResponse, Tree};
use iced::widget::image::Handle;
use iced::window;
use iced_layershell::to_layer_message;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::components::action_panel;

#[to_layer_message(multi)]
#[derive(Clone, Debug)]
pub enum Message {
    UpdateTree(Tree),
    SearchTextChanged(String),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    EscapePressed,
    ImageLoaded(String, Handle),
    ImageLoadFailed(String),
    InvokeAction(ActionHandler),

    ActionPanel(action_panel::Message),

    ShowToast(String),
    DropdownChanged(String),
    LaunchCommand(ExtensionCommand),
    ExtensionLaunched(Result<(SidecarWriter, Arc<Mutex<SidecarReader>>), String>),
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
    HandleOAuthRedirect(String),

    Tick(Instant),

    Root(crate::screens::root::RootMessage),
    Grid(crate::screens::grid::GridMessage),
    Detail(crate::screens::detail::DetailMessage),
    List(crate::screens::list::ListMessage),

    SidecarMessage(SidecarResponse),
    SidecarOperationFinished(u32, Result<Option<serde_json::Value>, String>),
    RunCallback(String, serde_json::Value),
}
