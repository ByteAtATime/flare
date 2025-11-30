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
            },
            Task::none(),
        )
    }
}

fn update(state: &mut State, message: SettingsMessage) -> Task<SettingsMessage> {
    if let SettingsMessage::TabChanged(tab) = message {
        state.current_tab = tab;
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
