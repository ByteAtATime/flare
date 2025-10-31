pub mod grid;
pub mod types;

use iced::Element;

use crate::Message;
use crate::types::TreeNode;
use types::Component;

pub fn render_tree_node(node: &TreeNode, selected_index: usize) -> Element<'_, Message> {
    match Component::from_tree_node(node) {
        Component::Grid(props) => grid::render_grid(props, selected_index),
        Component::GridSection(props) => grid::render_grid_section(props, selected_index),
        Component::GridItem(props) => grid::render_grid_item(props, false),
        Component::Unknown => iced::widget::text("Unknown").into(),
    }
}
