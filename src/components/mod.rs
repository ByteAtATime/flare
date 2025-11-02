pub mod actions;
pub mod column;
pub mod footer;
pub mod grid;
pub mod types;

use iced::Element;

use crate::{Message, position};
pub use types::Component;

pub fn render_component<'a>(
    component: &'a Component,
    selected_index: usize,
    column_id: position::Id,
) -> Element<'a, Message> {
    match component {
        Component::Grid(props) => grid::render_grid(props.clone(), selected_index, column_id),
        Component::GridItem(props) => grid::render_grid_item(props.clone(), false),
        _ => iced::widget::text("Unknown").into(),
    }
}
