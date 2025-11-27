use iced::{
    Color, Element, Length, Task,
    widget::{Button, column, container, mouse_area, opaque, row, text},
};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

use crate::{
    Message,
    components::types::{CallbackInfo, deserialize_icon},
    icons,
};

const ICON_FONT: iced::Font = iced::Font::with_name("Raycast-Icons");

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
            // Default handler: call back to the sidecar via IPC
            ActionHandler::new(move || {
                crate::globals::send_callback(cb.id.clone(), Value::Null);
                Task::none()
            })
        });

        Self {
            title: dto.props.title,
            icon: dto.props.icon,
            handler,
        }
    }
}

fn render_action(action: &Action) -> iced::Element<'_, crate::Message> {
    let mut button = Button::new(
        if let Some(icon) = action
            .icon
            .as_ref()
            .and_then(|icon_name| icons::get_icon(icon_name))
        {
            row![text(icon).font(ICON_FONT), text(action.title.clone())].into()
        } else {
            Element::from(text(action.title.clone()))
        },
    );

    if let Some(handler) = &action.handler {
        button = button.on_press(Message::InvokeAction(handler.clone()));
    }

    button.into()
}

pub fn render_action_panel(state: &crate::State) -> iced::Element<'_, crate::Message> {
    let actions = state
        .selected_actions
        .iter()
        .fold(column![].spacing(10), |col, action| {
            col.push({
                match action {
                    ActionPanelItem::Action(action) => render_action(action),
                    ActionPanelItem::Section(section) => {
                        let section_title = text(&section.props.title)
                            .size(16)
                            .color(Color::from_rgb8(0xFF, 0xFF, 0xFF));

                        let section_actions = section
                            .children
                            .iter()
                            .fold(column![].spacing(5), |col, action| {
                                col.push(render_action(action))
                            });

                        column![section_title, section_actions].spacing(5).into()
                    }
                }
            })
        });

    opaque(
        mouse_area(
            container(
                column![
                    container(opaque(actions))
                        .padding(8)
                        .style(|_theme| container::Style {
                            background: Some(Color::from_rgba(0.1, 0.1, 0.1, 0.95).into()),
                            ..Default::default()
                        }),
                    container(column![]).height(40)
                ]
                .spacing(10),
            )
            .align_bottom(Length::Fill)
            .align_right(Length::Fill),
        )
        .on_press(Message::ToggleActionPanel(false)),
    )
    .into()
}
