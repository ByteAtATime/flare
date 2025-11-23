use crate::components::types::ActionPanel;

pub mod detail;
pub mod grid;

pub trait Shell {
    fn can_search(&self) -> bool;
    fn on_search(&mut self, query: &str);

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel>;
}

pub enum Screen {
    Grid(grid::GridScreen),
    Detail(detail::DetailScreen),
}

impl Shell for Screen {
    fn can_search(&self) -> bool {
        match self {
            Screen::Grid(screen) => screen.can_search(),
            Screen::Detail(screen) => screen.can_search(),
        }
    }

    fn on_search(&mut self, query: &str) {
        match self {
            Screen::Grid(screen) => screen.on_search(query),
            Screen::Detail(screen) => screen.on_search(query),
        }
    }

    fn get_action_panel(&mut self) -> Option<&mut ActionPanel> {
        match self {
            Screen::Grid(screen) => screen.get_action_panel(),
            Screen::Detail(screen) => screen.get_action_panel(),
        }
    }
}
