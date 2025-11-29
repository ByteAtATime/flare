use iced::widget::Id as WidgetId;
use iced::window;
use std::path::PathBuf;
use std::time::Instant;

use crate::apps;
use crate::components::actions::ActionPanelItem;
use crate::extensions;
use crate::frecency::FrecencyStore;
use crate::preferences::PreferenceStore;
use crate::screens::{Screen, Shell};
use crate::theme::Theme;

pub struct State {
    pub screen: Screen,
    pub extensions: Vec<extensions::Extension>,
    pub apps: Vec<apps::AppEntry>,
    pub preferences: PreferenceStore,
    pub frecency: FrecencyStore,
    pub theme: Theme,
    pub search_text: String,
    pub search_input_id: WidgetId,
    pub action_panel_visible: bool,
    pub action_panel_search: String,
    pub action_panel_selected: usize,
    pub selected_actions: Vec<ActionPanelItem>,
    pub toast_message: String,
    pub window_id: Option<window::Id>,
    pub settings_window_id: Option<window::Id>,

    // Animation state
    pub action_panel_start_time: Option<Instant>,
    pub action_panel_opacity: f32,
    pub action_panel_scale: f32,
}

fn get_frecency_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("flare").join("frecency.json");
    }
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".flare").join("frecency.json");
    }
    PathBuf::from("frecency.json")
}

impl State {
    pub fn new() -> Self {
        let extensions = extensions::scan_extensions();
        let commands = extensions::get_launchable_commands(&extensions);
        let apps = apps::scan_applications();
        let preferences = PreferenceStore::load();
        let frecency = FrecencyStore::new(get_frecency_path());
        let theme = Theme::default();

        let mut search_text = String::new();
        if let Some(url) = crate::deep_link::get_current() {
            search_text = url;
        }

        let mut root_screen = crate::screens::root::RootScreen::new(commands, apps.clone());
        root_screen.sort_items(&frecency);

        let mut state = Self {
            screen: Screen::Root(root_screen),
            extensions,
            apps,
            preferences,
            frecency,
            theme,
            search_text,
            search_input_id: WidgetId::unique(),
            action_panel_visible: false,
            action_panel_search: String::new(),
            action_panel_selected: 0,
            selected_actions: Vec::new(),
            toast_message: String::new(),
            window_id: None,
            settings_window_id: None,
            action_panel_start_time: None,
            action_panel_opacity: 1.0,
            action_panel_scale: 1.0,
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
