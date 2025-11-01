pub mod actions;
pub mod footer;
pub mod grid;
pub mod types;

use iced::Element;

use crate::Message;
pub use types::Component;

pub fn render_component(component: &Component, selected_index: usize) -> Element<'_, Message> {
    match component {
        Component::Grid(props) => grid::render_grid(props.clone(), selected_index),
        Component::GridItem(props) => grid::render_grid_item(props.clone(), false),
        _ => iced::widget::text("Unknown").into(),
    }
}
