use crate::components::actions::ActionPanelItem;
use crate::extensions;
use crate::screens::{Screen, Shell};

pub struct State {
    pub screen: Screen,
    pub extensions: Vec<extensions::Extension>,
    pub search_text: String,
    pub action_panel_visible: bool,
    pub selected_actions: Vec<ActionPanelItem>,
    pub toast_message: String,
}

impl State {
    pub fn new() -> Self {
        let extensions = extensions::scan_extensions();
        let commands = extensions::get_launchable_commands(&extensions);

        Self {
            screen: Screen::Root(crate::screens::root::RootScreen::new(commands)),
            extensions,
            search_text: String::new(),
            action_panel_visible: false,
            selected_actions: Vec::new(),
            toast_message: String::new(),
        }
    }

    pub fn update_selected_actions(&mut self) {
        if let Some(action_panel) = self.screen.get_action_panel() {
            self.selected_actions = action_panel.children.clone();
        } else {
            self.selected_actions.clear();
        }
    }
}
