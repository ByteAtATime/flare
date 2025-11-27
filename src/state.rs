use iced::window;

use crate::apps;
use crate::components::actions::ActionPanelItem;
use crate::extensions;
use crate::preferences::PreferenceStore;
use crate::screens::{Screen, Shell};

pub struct State {
    pub screen: Screen,
    pub extensions: Vec<extensions::Extension>,
    pub apps: Vec<apps::AppEntry>,
    pub preferences: PreferenceStore,
    pub search_text: String,
    pub action_panel_visible: bool,
    pub selected_actions: Vec<ActionPanelItem>,
    pub toast_message: String,
    pub window_id: Option<window::Id>,
    pub settings_window_id: Option<window::Id>,
}

impl State {
    pub fn new() -> Self {
        let extensions = extensions::scan_extensions();
        let commands = extensions::get_launchable_commands(&extensions);
        let apps = apps::scan_applications();
        let preferences = PreferenceStore::load();

        let mut search_text = String::new();
        if let Some(url) = crate::deep_link::get_current() {
            search_text = url;
        }

        let mut state = Self {
            screen: Screen::Root(crate::screens::root::RootScreen::new(
                commands,
                apps.clone(),
            )),
            extensions,
            apps,
            preferences,
            search_text,
            action_panel_visible: false,
            selected_actions: Vec::new(),
            toast_message: String::new(),
            window_id: None,
            settings_window_id: None,
        };

        state.update_selected_actions();

        state
    }

    pub fn update_selected_actions(&mut self) {
        if let Some(action_panel) = self.screen.get_action_panel() {
            self.selected_actions = action_panel.children.clone();
        } else {
            self.selected_actions.clear();
        }
    }
}
