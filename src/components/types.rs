use iced::Color;
use rustyscript::serde_json::Value;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum Component {
    Grid(GridProps),
    GridSection(GridSectionProps),
    GridItem(GridItemProps),
    Unknown,
}

mod raw {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct TreeNode {
        #[serde(rename = "type")]
        pub node_type: String,
        pub props: Option<Value>,
        pub children: Vec<TreeNode>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ActionPanel {
        pub children: Vec<Value>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ActionProps {
        pub title: String,
        pub icon: Option<String>,
        #[serde(rename = "onAction")]
        pub on_action: Option<super::CallbackInfo>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Action {
        pub props: ActionProps,
    }

    #[derive(Deserialize, Debug)]
    pub struct SectionProps {
        pub title: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Section {
        pub props: SectionProps,
        pub children: Vec<Value>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "type")]
    pub enum ActionItem {
        #[serde(rename = "ActionPanel.Section")]
        Section(Section),
        Action(Action),
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct GridProps {
    #[serde(default)]
    pub columns: Option<i32>,
    #[serde(default, rename = "onSearchTextChange")]
    pub on_search_text_change: Option<CallbackInfo>,
    #[serde(skip)]
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
    #[serde(default, deserialize_with = "deserialize_content")]
    pub content: Option<GridItemContent>,
    #[serde(default, skip)]
    pub actions: Option<ActionPanel>,
}

#[derive(Debug, Clone)]
pub enum GridItemContent {
    Color(GridItemColor),
    Image(String),
}

fn deserialize_content<'de, D>(deserializer: D) -> Result<Option<GridItemContent>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::String(s) => Ok(Some(GridItemContent::Image(s))),
        Value::Object(map) => {
            if let Some(color_value) = map.get("color") {
                let color: GridItemColor =
                    serde_json::from_value(color_value.clone()).map_err(D::Error::custom)?;
                Ok(Some(GridItemContent::Color(color)))
            } else {
                Ok(None)
            }
        }
        Value::Null => Ok(None),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GridItemColor {
    pub light: String,
    pub dark: String,
    #[serde(rename = "adjustContrast")]
    pub adjust_contrast: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ActionPanel {
    pub children: Vec<ActionPanelItem>,
}

#[derive(Debug, Clone)]
pub enum ActionPanelItem {
    Section(ActionPanelSection),
    Action(Action),
}

#[derive(Debug, Clone, Default)]
pub struct ActionPanelSection {
    pub title: String,
    pub children: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub title: String,
    // TODO: support all "ImageLike" types
    pub icon: Option<String>,
    pub on_action: Option<CallbackInfo>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CallbackInfo {
    #[serde(rename = "type")]
    pub callback_type: String,
    pub id: String,
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

fn parse_action_panel(value: &Value) -> Option<ActionPanel> {
    let action_panel: raw::ActionPanel = serde_json::from_value(value.clone()).ok()?;

    Some(ActionPanel {
        children: action_panel
            .children
            .into_iter()
            .filter_map(parse_action_item)
            .collect(),
    })
}

fn parse_action_item(value: Value) -> Option<ActionPanelItem> {
    let raw_item: raw::ActionItem = serde_json::from_value(value).ok()?;

    match raw_item {
        raw::ActionItem::Section(section) => Some(ActionPanelItem::Section(ActionPanelSection {
            title: section.props.title,
            children: section
                .children
                .into_iter()
                .filter_map(parse_action_item)
                .filter_map(|item| match item {
                    ActionPanelItem::Action(action) => Some(action),
                    _ => None,
                })
                .collect(),
        })),
        raw::ActionItem::Action(action) => Some(ActionPanelItem::Action(Action {
            title: action.props.title,
            icon: action.props.icon,
            on_action: action.props.on_action,
        })),
    }
}

impl Component {
    fn from_raw_node(node: raw::TreeNode) -> Self {
        match node.node_type.as_str() {
            "Grid" => {
                let mut props: GridProps = parse_props(&node.props);
                props.sections = node
                    .children
                    .into_iter()
                    .filter_map(|child| match Self::from_raw_node(child) {
                        Component::GridSection(section) => Some(section),
                        _ => None,
                    })
                    .collect();

                Component::Grid(props)
            }
            "Grid.Section" => {
                let mut props: GridSectionProps = parse_props(&node.props);
                props.items = node
                    .children
                    .into_iter()
                    .filter_map(|child| match Self::from_raw_node(child) {
                        Component::GridItem(item) => Some(item),
                        _ => None,
                    })
                    .collect();

                Component::GridSection(props)
            }
            "Grid.Item" => {
                let mut props: GridItemProps = parse_props(&node.props);

                if let Some(Value::Object(map)) = node.props.as_ref() {
                    if let Some(actions_value) = map.get("actions") {
                        props.actions = parse_action_panel(actions_value);
                    }
                }
                Component::GridItem(props)
            }
            _ => Component::Unknown,
        }
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_node = raw::TreeNode::deserialize(deserializer)?;
        Ok(Component::from_raw_node(raw_node))
    }
}
