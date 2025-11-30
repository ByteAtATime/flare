use iced::widget::container;
use iced::{Element, Length, Task};

use crate::extensions::{Extension, scan_extensions};
use crate::preferences::{FlareSettings, PreferenceStore};
use crate::theme::Theme;

use super::view::settings_view;
use super::{SettingsMessage, SettingsTab, handle_message};

struct State {
    extensions: Vec<Extension>,
    preferences: PreferenceStore,
    flare_settings: FlareSettings,
    theme: Theme,
    current_tab: SettingsTab,
    selected_extension: Option<usize>,
    pressed_extension: Option<usize>,
    extension_search: String,
}

impl State {
    fn new() -> (Self, Task<SettingsMessage>) {
        (
            Self {
                extensions: scan_extensions(),
                preferences: PreferenceStore::load(),
                flare_settings: FlareSettings::load(),
                theme: Theme::default(),
                current_tab: SettingsTab::default(),
                selected_extension: None,
                pressed_extension: None,
                extension_search: String::new(),
            },
            Task::none(),
        )
    }
}

fn update(state: &mut State, message: SettingsMessage) -> Task<SettingsMessage> {
    match &message {
        SettingsMessage::TabChanged(tab) => state.current_tab = *tab,
        SettingsMessage::ExtensionSelected(idx) => {
            state.selected_extension = Some(*idx);
            state.pressed_extension = None;
        }
        SettingsMessage::ExtensionPressed(idx) => state.pressed_extension = *idx,
        SettingsMessage::ExtensionSearchChanged(query) => state.extension_search = query.clone(),
        _ => {}
    }
    handle_message(&message, &mut state.preferences, &mut state.flare_settings);
    Task::none()
}

fn view(state: &State) -> Element<'_, SettingsMessage> {
    let bg_color = state.theme.colors.background;

    container(settings_view(
        &state.extensions,
        &state.preferences,
        &state.flare_settings,
        &state.theme,
        state.current_tab,
        state.selected_extension,
        state.pressed_extension,
        &state.extension_search,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        ..Default::default()
    })
    .into()
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    iced::application(State::new, update, view)
        .title("Flare Settings")
        .font(include_bytes!("../assets/Inter.ttf").as_slice())
        .font(include_bytes!("../assets/icons.ttf").as_slice())
        .default_font(iced::Font::DEFAULT)
        .run()
        .map_err(|e| e.to_string())?;

    Ok(())
}
