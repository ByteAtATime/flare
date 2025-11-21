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
    #[serde(tag = "type")]
    pub enum TreeNode {
        Grid {
            props: Option<GridProps>,
            children: Vec<TreeNode>,
        },
        #[serde(rename = "Grid.Section")]
        GridSection {
            props: Option<GridSectionProps>,
            children: Vec<TreeNode>,
        },
        #[serde(rename = "Grid.Item")]
        GridItem {
            props: Option<RawGridItemProps>,
        },
        #[serde(other)]
        Unknown,
    }

    #[derive(Debug, Default, Deserialize)]
    pub struct RawGridItemProps {
        #[serde(default)]
        pub title: String,
        #[serde(default)]
        pub subtitle: Option<String>,
        #[serde(default, deserialize_with = "super::deserialize_content")]
        pub content: Option<super::GridItemContent>,
        #[serde(default)]
        pub actions: Option<Value>,
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

impl From<raw::Action> for Action {
    fn from(raw: raw::Action) -> Self {
        Self {
            title: raw.props.title,
            icon: raw.props.icon,
            on_action: raw.props.on_action,
        }
    }
}

impl From<raw::Section> for ActionPanelSection {
    fn from(raw: raw::Section) -> Self {
        Self {
            title: raw.props.title,
            children: raw
                .children
                .into_iter()
                .filter_map(|v| serde_json::from_value::<raw::ActionItem>(v).ok())
                .filter_map(|item| match item {
                    raw::ActionItem::Action(action) => Some(action.into()),
                    _ => None,
                })
                .collect(),
        }
    }
}

impl TryFrom<Value> for ActionPanelItem {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let raw_item = serde_json::from_value::<raw::ActionItem>(value)?;
        Ok(match raw_item {
            raw::ActionItem::Section(section) => ActionPanelItem::Section(section.into()),
            raw::ActionItem::Action(action) => ActionPanelItem::Action(action.into()),
        })
    }
}

impl TryFrom<Value> for ActionPanel {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let raw = serde_json::from_value::<raw::ActionPanel>(value)?;
        Ok(Self {
            children: raw
                .children
                .into_iter()
                .filter_map(|v| v.try_into().ok())
                .collect(),
        })
    }
}

impl Component {
    fn from_raw_node(node: raw::TreeNode) -> Self {
        match node {
            raw::TreeNode::Grid { props, children } => {
                let mut props = props.unwrap_or_default();
                props.sections = children
                    .into_iter()
                    .filter_map(|child| match Self::from_raw_node(child) {
                        Component::GridSection(section) => Some(section),
                        _ => None,
                    })
                    .collect();
                Component::Grid(props)
            }
            raw::TreeNode::GridSection { props, children } => {
                let mut props = props.unwrap_or_default();
                props.items = children
                    .into_iter()
                    .filter_map(|child| match Self::from_raw_node(child) {
                        Component::GridItem(item) => Some(item),
                        _ => None,
                    })
                    .collect();
                Component::GridSection(props)
            }
            raw::TreeNode::GridItem { props } => {
                let raw_props = props.unwrap_or_default();
                Component::GridItem(GridItemProps {
                    title: raw_props.title,
                    subtitle: raw_props.subtitle,
                    content: raw_props.content,
                    actions: raw_props.actions.and_then(|v| v.try_into().ok()),
                })
            }
            raw::TreeNode::Unknown => Component::Unknown,
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
