pub mod grid;

use iced::Element;

use crate::Message;
use crate::types::TreeNode;

pub fn render_tree_node(node: &TreeNode, selected_index: usize) -> Element<'_, Message> {
    match node.node_type.as_str() {
        "Grid" => grid::render_grid(node, selected_index),
        "Grid.Section" => grid::render_grid_section(node, selected_index),
        "Grid.Item" => grid::render_grid_item(node, false),
        _ => iced::widget::text("Unknown").into(),
    }
}
