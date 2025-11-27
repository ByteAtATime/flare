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
struct RawThemeColors {
    #[serde(deserialize_with = "deserialize_color")]
    background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    background_secondary: Color,
    #[serde(deserialize_with = "deserialize_color")]
    text: Color,
    #[serde(deserialize_with = "deserialize_color")]
    selection: Color,
    #[serde(deserialize_with = "deserialize_color")]
    loader: Color,
    #[serde(deserialize_with = "deserialize_color")]
    red: Color,
    #[serde(deserialize_with = "deserialize_color")]
    orange: Color,
    #[serde(deserialize_with = "deserialize_color")]
    yellow: Color,
    #[serde(deserialize_with = "deserialize_color")]
    green: Color,
    #[serde(deserialize_with = "deserialize_color")]
    blue: Color,
    #[serde(deserialize_with = "deserialize_color")]
    purple: Color,
    #[serde(deserialize_with = "deserialize_color")]
    magenta: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: Color,
    pub background_secondary: Color,
    pub text: Color,
    pub selection: Color,
    pub loader: Color,
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub blue: Color,
    pub purple: Color,
    pub magenta: Color,
    pub background_primary_40: Color,
    pub background_secondary_40: Color,
    pub border_10: Color,
    pub border_20: Color,
    pub text_10: Color,
    pub text_40: Color,
    pub text_60: Color,
    pub selection_10: Color,
    pub green_10: Color,
    pub yellow_10: Color,
    pub red_10: Color,
    pub orange_10: Color,
    pub blue_10: Color,
    pub purple_10: Color,
    pub magenta_10: Color,
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

impl From<RawThemeColors> for ThemeColors {
    fn from(raw: RawThemeColors) -> Self {
        Self {
            background: raw.background,
            background_secondary: raw.background_secondary,
            text: raw.text,
            selection: raw.selection,
            loader: raw.loader,
            red: raw.red,
            orange: raw.orange,
            yellow: raw.yellow,
            green: raw.green,
            blue: raw.blue,
            purple: raw.purple,
            magenta: raw.magenta,
            background_primary_40: with_alpha(raw.background, 0.40),
            background_secondary_40: with_alpha(raw.background_secondary, 0.40),
            border_10: with_alpha(raw.text, 0.10),
            border_20: with_alpha(raw.text, 0.20),
            text_10: with_alpha(raw.text, 0.10),
            text_40: with_alpha(raw.text, 0.40),
            text_60: with_alpha(raw.text, 0.60),
            selection_10: with_alpha(raw.selection, 0.10),
            green_10: with_alpha(raw.green, 0.15),
            yellow_10: with_alpha(raw.yellow, 0.15),
            red_10: with_alpha(raw.red, 0.15),
            orange_10: with_alpha(raw.orange, 0.15),
            blue_10: with_alpha(raw.blue, 0.15),
            purple_10: with_alpha(raw.purple, 0.15),
            magenta_10: with_alpha(raw.magenta, 0.15),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        let raw = RawThemeColors {
            background: color!(0x1b1b1b),
            background_secondary: color!(0x000000),
            text: color!(0xf2f2f2),
            selection: color!(0x323232),
            loader: color!(0x000000),
            red: color!(0xf84e4e),
            orange: color!(0xf88d4e),
            yellow: color!(0xffcc47),
            green: color!(0x4ef8a7),
            blue: color!(0x228cf6),
            purple: color!(0x7b4ef8),
            magenta: color!(0xf84ebd),
        };
        Self {
            name: "Raycast Dark".to_string(),
            appearance: "dark".to_string(),
            colors: raw.into(),
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
            #[serde(flatten)]
            colors: RawThemeColors,
        }

        let helper = ThemeHelper::deserialize(deserializer)?;
        Ok(Theme {
            name: helper.name,
            appearance: helper.appearance,
            colors: helper.colors.into(),
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
