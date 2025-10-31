pub mod grid;

use iced::Element;

use crate::Message;
use crate::types::TreeNode;

pub fn render_tree_node(node: &TreeNode) -> Element<'_, Message> {
    match node.node_type.as_str() {
        "Grid" => grid::render_grid(node),
        "Grid.Section" => grid::render_grid_section(node),
        "Grid.Item" => grid::render_grid_item(node),
        _ => iced::widget::text("Unknown").into(),
    }
}
