use iced::widget::{column, container, row, text};
use iced::{Color, Element, Theme};
use rustyscript::serde_json::Value;

use crate::Message;
use crate::types::TreeNode;

const INTER_FONT: iced::Font = iced::Font::with_name("Inter");

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct GridSectionProps {
    #[serde(default)]
    title: String,
    #[serde(default)]
    columns: Option<i32>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct GridItemProps {
    #[serde(default)]
    title: String,
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    content: Option<GridItemContent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GridItemContent {
    color: Option<GridItemColor>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GridItemColor {
    light: String,
    dark: String,
    #[serde(rename = "adjustContrast")]
    adjust_contrast: bool,
}

fn parse_hex_color(hex: &str) -> Color {
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

pub fn render_grid(node: &TreeNode) -> Element<'_, Message> {
    node.children
        .iter()
        .fold(column![], |col, child| col.push(render_grid_section(child)))
        .into()
}

pub fn render_grid_section(node: &TreeNode) -> Element<'_, Message> {
    let props: GridSectionProps = parse_props(&node.props);
    let items_row = node.children.iter().fold(row![].spacing(10), |row, child| {
        row.push(render_grid_item(child))
    });

    column![text(props.title).size(16).font(INTER_FONT), items_row]
        .padding(10)
        .spacing(10)
        .into()
}

pub fn render_grid_item(node: &TreeNode) -> Element<'_, Message> {
    let props: GridItemProps = parse_props(&node.props);
    let bg_color = props
        .content
        .as_ref()
        .and_then(|c| c.color.as_ref())
        .map(|color| parse_hex_color(&color.light))
        .unwrap_or(Color::from_rgb8(0x33, 0x33, 0x33));

    container(
        column![
            container(text(""))
                .width(150)
                .height(150)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(bg_color.into()),
                    border: iced::Border {
                        color: Color::from_rgb8(0x55, 0x55, 0x55),
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            text(props.title).size(14).font(INTER_FONT),
            text(props.subtitle.unwrap_or_default())
                .size(12)
                .font(INTER_FONT)
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb8(0xaa, 0xaa, 0xaa)),
                    ..Default::default()
                })
        ]
        .spacing(5),
    )
    .into()
}
