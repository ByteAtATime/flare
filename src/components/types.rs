use iced::Color;
use rustyscript::serde_json::Value;

#[derive(Debug, Clone)]
pub enum Component {
    Grid(GridProps),
    GridSection(GridSectionProps),
    GridItem(GridItemProps),
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct GridProps {
    pub sections: Vec<GridSectionProps>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct GridSectionProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub columns: Option<i32>,
    #[serde(skip)]
    pub items: Vec<GridItemProps>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct GridItemProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub content: Option<GridItemContent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GridItemContent {
    pub color: Option<GridItemColor>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GridItemColor {
    pub light: String,
    pub dark: String,
    #[serde(rename = "adjustContrast")]
    pub adjust_contrast: bool,
}

pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

fn parse_props<T: serde::de::DeserializeOwned + Default>(props: &Option<Value>) -> T {
    props
        .as_ref()
        .and_then(|p| serde_json::from_value(p.clone()).ok())
        .unwrap_or_default()
}

impl Component {
    pub fn from_tree_node(node: &crate::types::TreeNode) -> Self {
        match node.node_type.as_str() {
            "Grid" => {
                let sections = node
                    .children
                    .iter()
                    .filter_map(|child| {
                        if let Component::GridSection(section) = Self::from_tree_node(child) {
                            Some(section)
                        } else {
                            None
                        }
                    })
                    .collect();

                Component::Grid(GridProps { sections })
            }
            "Grid.Section" => {
                let mut props: GridSectionProps = parse_props(&node.props);
                props.items = node
                    .children
                    .iter()
                    .filter_map(|child| {
                        if let Component::GridItem(item) = Self::from_tree_node(child) {
                            Some(item)
                        } else {
                            None
                        }
                    })
                    .collect();

                Component::GridSection(props)
            }
            "Grid.Item" => Component::GridItem(parse_props(&node.props)),
            _ => Component::Unknown,
        }
    }
}
