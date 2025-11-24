use iced::Color;
use serde::Deserialize;
use serde_json::Value;

pub use super::detail::DetailProps;
pub use super::grid::{GridItemProps, GridProps, GridSectionProps};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    Grid(GridProps),
    #[serde(rename = "Grid.Section")]
    GridSection(GridSectionProps),
    #[serde(rename = "Grid.Item")]
    GridItem(GridItemProps),
    Detail(DetailProps),
    #[serde(other)]
    Unknown,
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

pub fn deserialize_icon<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Value = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(Some(s)),
        Value::Object(map) => {
            if let Some(Value::String(s)) = map.get("source") {
                Ok(Some(s.clone()))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}
