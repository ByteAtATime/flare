use iced::Task;
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

use crate::{
    Message,
    components::types::{CallbackInfo, deserialize_icon},
};

#[derive(Clone)]
pub struct ActionHandler(Arc<dyn Fn() -> Task<Message> + Send + Sync>);

impl ActionHandler {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> Task<Message> + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }

    pub fn call(&self) -> Task<Message> {
        (self.0)()
    }
}

impl fmt::Debug for ActionHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ActionHandler")
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(from = "ActionPanelDto")]
pub struct ActionPanel {
    pub children: Vec<ActionPanelItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ActionPanelItem {
    #[serde(rename = "ActionPanel.Section")]
    Section(ActionPanelSection),
    Action(Action),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActionPanelSection {
    #[serde(default)]
    pub props: ActionPanelSectionProps,
    #[serde(default)]
    pub children: Vec<Action>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActionPanelSectionProps {
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub title: String,
    pub icon: Option<String>,
    pub handler: Option<ActionHandler>,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto = ActionDto::deserialize(deserializer)?;
        Ok(Action::from(dto))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ActionPanelDto {
    #[serde(default)]
    pub children: Vec<ActionPanelItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct ActionDto {
    #[serde(default)]
    pub props: ActionPropsDto,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ActionPropsDto {
    #[serde(default)]
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_icon")]
    pub icon: Option<String>,
    #[serde(rename = "onAction")]
    pub on_action: Option<CallbackInfo>,
}

impl From<ActionPanelDto> for ActionPanel {
    fn from(dto: ActionPanelDto) -> Self {
        Self {
            children: dto.children,
        }
    }
}

impl From<ActionDto> for Action {
    fn from(dto: ActionDto) -> Self {
        let handler = dto.props.on_action.map(|cb| {
            ActionHandler::new(move || Task::done(Message::RunCallback(cb.id.clone(), Value::Null)))
        });

        Self {
            title: dto.props.title,
            icon: dto.props.icon,
            handler,
        }
    }
}
