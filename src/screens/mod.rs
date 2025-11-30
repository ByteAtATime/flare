use crate::components::actions::ActionPanel;
use crate::components::types::{CallbackInfo, Component};
use iced::Task;

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

    fn set_dropdown_value(&mut self, _value: &str) {}

    fn on_search_text_change(&self) -> Option<&CallbackInfo> {
        None
    }

    fn load_images(&self) -> Task<crate::Message> {
        Task::none()
    }
}

pub enum Screen {
    Root(root::RootScreen),
    Grid(grid::GridScreen),
    Detail(detail::DetailScreen),
    List(list::ListScreen),
}

impl Screen {
    pub fn new(component: Component, previous: Option<&Screen>) -> Option<Self> {
        let (viewport, scroll_id, acc_value) = match previous {
            Some(prev) => (
                match prev {
                    Screen::Grid(s) => s.get_viewport(),
                    Screen::List(s) => s.get_viewport(),
                    _ => None,
                },
                match prev {
                    Screen::Grid(s) => Some(s.scrollable_id.clone()),
                    Screen::List(s) => Some(s.scrollable_id.clone()),
                    _ => None,
                },
                prev.get_search_bar_accessory()
                    .and_then(|d| d.props.value.clone()),
            ),
            None => (None, None, None),
        };

        match component {
            Component::Grid(mut props) => {
                if let Some(acc) = props.props.search_bar_accessory.as_mut() {
                    if acc.props.value.is_none() {
                        acc.props.value = acc_value;
                    }
                }
                Some(Screen::Grid(grid::GridScreen::new(
                    props, viewport, scroll_id,
                )))
            }
            Component::List(mut props) => {
                if let Some(acc) = props.props.search_bar_accessory.as_mut() {
                    if acc.props.value.is_none() {
                        acc.props.value = acc_value;
                    }
                }
                Some(Screen::List(list::ListScreen::new(
                    props, viewport, scroll_id,
                )))
            }
            Component::Detail(props) => Some(Screen::Detail(detail::DetailScreen::new(props))),
            _ => None,
        }
    }
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

    fn on_search_text_change(&self) -> Option<&CallbackInfo> {
        match self {
            Screen::Root(screen) => screen.on_search_text_change(),
            Screen::Grid(screen) => screen.on_search_text_change(),
            Screen::Detail(screen) => screen.on_search_text_change(),
            Screen::List(screen) => screen.on_search_text_change(),
        }
    }

    fn load_images(&self) -> Task<crate::Message> {
        match self {
            Screen::Root(screen) => screen.load_images(),
            Screen::Grid(screen) => screen.load_images(),
            Screen::Detail(screen) => screen.load_images(),
            Screen::List(screen) => screen.load_images(),
        }
    }
}
