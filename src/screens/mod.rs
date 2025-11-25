use crate::components::actions::ActionPanel;

pub mod detail;
pub mod grid;
pub mod list;
pub mod root;

pub trait Shell {
    fn can_search(&self) -> bool;
    fn on_search(&mut self, query: &str);

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel>;

    fn get_search_bar_accessory(&self) -> Option<&crate::components::dropdown::Dropdown> {
        None
    }

    fn set_dropdown_value(&mut self, _value: &str) {
    }
}

pub enum Screen {
    Root(root::RootScreen),
    Grid(grid::GridScreen),
    Detail(detail::DetailScreen),
    List(list::ListScreen),
}

impl Shell for Screen {
    fn can_search(&self) -> bool {
        match self {
            Screen::Root(screen) => screen.can_search(),
            Screen::Grid(screen) => screen.can_search(),
            Screen::Detail(screen) => screen.can_search(),
            Screen::List(screen) => screen.can_search(),
        }
    }

    fn on_search(&mut self, query: &str) {
        match self {
            Screen::Root(screen) => screen.on_search(query),
            Screen::Grid(screen) => screen.on_search(query),
            Screen::Detail(screen) => screen.on_search(query),
            Screen::List(screen) => screen.on_search(query),
        }
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        match self {
            Screen::Root(screen) => screen.get_action_panel(),
            Screen::Grid(screen) => screen.get_action_panel(),
            Screen::Detail(screen) => screen.get_action_panel(),
            Screen::List(screen) => screen.get_action_panel(),
        }
    }

    fn get_search_bar_accessory(&self) -> Option<&crate::components::dropdown::Dropdown> {
        match self {
            Screen::Root(screen) => screen.get_search_bar_accessory(),
            Screen::Grid(screen) => screen.get_search_bar_accessory(),
            Screen::Detail(screen) => screen.get_search_bar_accessory(),
            Screen::List(screen) => screen.get_search_bar_accessory(),
        }
    }

    fn set_dropdown_value(&mut self, value: &str) {
        match self {
            Screen::Root(screen) => screen.set_dropdown_value(value),
            Screen::Grid(screen) => screen.set_dropdown_value(value),
            Screen::Detail(screen) => screen.set_dropdown_value(value),
            Screen::List(screen) => screen.set_dropdown_value(value),
        }
    }
}
