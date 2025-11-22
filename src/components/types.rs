use iced::Color;
use rustyscript::serde_json::Value;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum Component {
    Grid(GridProps),
    GridSection(GridSectionProps),
    GridItem(GridItemProps),
    Detail(DetailProps),
    Unknown,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GridProps {
    #[serde(default)]
    pub columns: Option<i32>,
    #[serde(default, rename = "onSearchTextChange")]
    pub on_search_text_change: Option<CallbackInfo>,
    #[serde(skip)]
    pub sections: Vec<GridSectionProps>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GridSectionProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub columns: Option<i32>,
    #[serde(skip)]
    pub items: Vec<GridItemProps>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GridItemProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default, deserialize_with = "deserialize_content")]
    pub content: Option<GridItemContent>,
    #[serde(skip)]
    pub actions: Option<ActionPanel>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DetailProps {
    #[serde(default)]
    pub markdown: String,
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

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum RawComponent {
            Grid {
                #[serde(default)]
                props: Option<GridProps>,
                children: Vec<RawComponent>,
            },
            #[serde(rename = "Grid.Section")]
            GridSection {
                #[serde(default)]
                props: Option<GridSectionProps>,
                children: Vec<RawComponent>,
            },
            #[serde(rename = "Grid.Item")]
            GridItem {
                #[serde(default)]
                props: Option<RawGridItemProps>,
            },
            Detail {
                #[serde(default)]
                props: Option<DetailProps>,
            },
            #[serde(other)]
            Unknown,
        }

        #[derive(Default, Deserialize)]
        struct RawGridItemProps {
            #[serde(default)]
            title: String,
            #[serde(default)]
            subtitle: Option<String>,
            #[serde(default, deserialize_with = "deserialize_content")]
            content: Option<GridItemContent>,
            #[serde(default)]
            actions: Option<Value>,
        }

        fn convert(raw: RawComponent) -> Component {
            match raw {
                RawComponent::Grid { props, children } => {
                    let mut props = props.unwrap_or_default();
                    props.sections = children
                        .into_iter()
                        .filter_map(|c| match convert(c) {
                            Component::GridSection(s) => Some(s),
                            _ => None,
                        })
                        .collect();
                    Component::Grid(props)
                }
                RawComponent::GridSection { props, children } => {
                    let mut props = props.unwrap_or_default();
                    props.items = children
                        .into_iter()
                        .filter_map(|c| match convert(c) {
                            Component::GridItem(i) => Some(i),
                            _ => None,
                        })
                        .collect();
                    Component::GridSection(props)
                }
                RawComponent::GridItem { props } => {
                    let raw_props = props.unwrap_or_default();
                    Component::GridItem(GridItemProps {
                        title: raw_props.title,
                        subtitle: raw_props.subtitle,
                        content: raw_props.content,
                        actions: raw_props.actions.and_then(|v| parse_action_panel(&v)),
                    })
                }
                RawComponent::Detail { props } => {
                    Component::Detail(props.unwrap_or_default())
                }
                RawComponent::Unknown => Component::Unknown,
            }
        }

        let raw = RawComponent::deserialize(deserializer)?;
        Ok(convert(raw))
    }
}

fn parse_action_panel(value: &Value) -> Option<ActionPanel> {
    #[derive(Deserialize)]
    struct RawActionPanel {
        children: Vec<Value>,
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum RawActionItem {
        #[serde(rename = "ActionPanel.Section")]
        Section {
            props: RawSectionProps,
            children: Vec<Value>,
        },
        Action {
            props: RawActionProps,
        },
    }

    #[derive(Deserialize)]
    struct RawSectionProps {
        title: String,
    }

    #[derive(Deserialize)]
    struct RawActionProps {
        title: String,
        icon: Option<String>,
        #[serde(rename = "onAction")]
        on_action: Option<CallbackInfo>,
    }

    let panel: RawActionPanel = serde_json::from_value(value.clone()).ok()?;

    let children = panel
        .children
        .into_iter()
        .filter_map(|v| {
            let item: RawActionItem = serde_json::from_value(v).ok()?;
            match item {
                RawActionItem::Section { props, children } => {
                    Some(ActionPanelItem::Section(ActionPanelSection {
                        title: props.title,
                        children: children
                            .into_iter()
                            .filter_map(|v| {
                                if let Ok(RawActionItem::Action { props }) =
                                    serde_json::from_value(v)
                                {
                                    Some(Action {
                                        title: props.title,
                                        icon: props.icon,
                                        on_action: props.on_action,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    }))
                }
                RawActionItem::Action { props } => Some(ActionPanelItem::Action(Action {
                    title: props.title,
                    icon: props.icon,
                    on_action: props.on_action,
                })),
            }
        })
        .collect();

    Some(ActionPanel { children })
}
