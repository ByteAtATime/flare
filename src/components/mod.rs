// pub mod actions;
pub mod column;
// pub mod detail;
// pub mod footer;
pub mod grid;
pub mod types;

use iced::Element;
use iced::widget::scrollable::Viewport;

use crate::{Message, position};
pub use types::Component;

pub fn render_component<'a>(
    component: &'a Component,
    selected_index: usize,
    column_id: position::Id,
    viewport: Option<&Viewport>,
) -> Element<'a, Message> {
    match component {
        // Component::Grid(props) => {
        // grid::render_grid(props.clone(), selected_index, column_id, viewport)
        // }
        // Component::GridItem(props) => grid::render_grid_item(props.clone(), false, true),
        // Component::Detail(props) => detail::render_detail(&props),
        _ => iced::widget::text("Unknown").into(),
    }
}
