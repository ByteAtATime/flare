use iced::{Color, color};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub appearance: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    #[serde(deserialize_with = "deserialize_color")]
    pub background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub background_secondary: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub text: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub selection: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub loader: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub red: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub orange: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub yellow: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub green: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub blue: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub purple: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub magenta: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Raycast Dark".to_string(),
            appearance: "dark".to_string(),
            colors: ThemeColors {
                background: color!(0x1b1b1b),
                // TODO: where is this color used?
                background_secondary: color!(0x000000),
                text: color!(0xf2f2f2),
                selection: color!(0x323232),
                // TODO
                loader: color!(0x000000),
                red: color!(0xf84e4e),
                orange: color!(0xf88d4e),
                yellow: color!(0xffcc47),
                green: color!(0x4ef8a7),
                blue: color!(0x228cf6),
                purple: color!(0x7b4ef8),
                magenta: color!(0xf84ebd),
            },
        }
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ThemeHelper {
            name: String,
            appearance: String,
            colors: ThemeColors,
        }

        let helper = ThemeHelper::deserialize(deserializer)?;
        Ok(Theme {
            name: helper.name,
            appearance: helper.appearance,
            colors: helper.colors,
        })
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Theme::default().colors
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_color(&s))
}

fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::BLACK;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    Color::from_rgb8(r, g, b)
}
