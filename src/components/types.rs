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

#[derive(Debug, Deserialize)]
struct RawTreeNode {
    #[serde(rename = "type")]
    node_type: String,
    props: Option<Value>,
    children: Vec<RawTreeNode>,
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
    pub children: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub title: String,
    pub on_action: Option<CallbackInfo>,
}

#[derive(Debug, Clone)]
pub struct CallbackInfo {
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
    if let Value::Object(map) = value {
        if let Some(Value::Array(children)) = map.get("children") {
            let actions: Vec<Action> = children
                .iter()
                .filter_map(|child| {
                    if let Value::Object(action_map) = child {
                        if let Some(Value::Object(props)) = action_map.get("props") {
                            let title = props
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let on_action = props.get("onAction").and_then(|v| {
                                if let Value::Object(callback_map) = v {
                                    let callback_type = callback_map
                                        .get("type")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let id = callback_map
                                        .get("id")
                                        .and_then(|i| i.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    if !callback_type.is_empty() && !id.is_empty() {
                                        return Some(CallbackInfo { callback_type, id });
                                    }
                                }
                                None
                            });

                            if !title.is_empty() {
                                return Some(Action { title, on_action });
                            }
                        }
                    }
                    None
                })
                .collect();

            if !actions.is_empty() {
                return Some(ActionPanel { children: actions });
            }
        }
    }
    None
}

impl Component {
    fn from_raw_node(node: RawTreeNode) -> Self {
        match node.node_type.as_str() {
            "Grid" => {
                let sections = node
                    .children
                    .into_iter()
                    .filter_map(|child| {
                        if let Component::GridSection(section) = Self::from_raw_node(child) {
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
                    .into_iter()
                    .filter_map(|child| {
                        if let Component::GridItem(item) = Self::from_raw_node(child) {
                            Some(item)
                        } else {
                            None
                        }
                    })
                    .collect();

                Component::GridSection(props)
            }
            "Grid.Item" => {
                let mut props: GridItemProps = parse_props(&node.props);

                if let Some(actions_value) = node.props.as_ref().and_then(|p| {
                    if let Value::Object(map) = p {
                        map.get("actions")
                    } else {
                        None
                    }
                }) {
                    props.actions = parse_action_panel(actions_value);
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
        let raw_node = RawTreeNode::deserialize(deserializer)?;
        Ok(Component::from_raw_node(raw_node))
    }
}
