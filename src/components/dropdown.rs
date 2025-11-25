use super::types::CallbackInfo;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DropdownChild {
    #[serde(rename = "Grid.Dropdown.Section")]
    GridSection(DropdownSection),
    #[serde(rename = "Grid.Dropdown.Item")]
    GridItem(DropdownItem),
    #[serde(rename = "List.Dropdown.Section")]
    ListSection(DropdownSection),
    #[serde(rename = "List.Dropdown.Item")]
    ListItem(DropdownItem),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Dropdown {
    pub props: DropdownProps,
    #[serde(default)]
    pub children: Vec<DropdownChild>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DropdownProps {
    pub tooltip: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default, rename = "defaultValue")]
    pub default_value: Option<String>,
    #[serde(default, rename = "onChange")]
    pub on_change: Option<CallbackInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DropdownSection {
    pub props: DropdownSectionProps,
    #[serde(default)]
    pub children: Vec<DropdownItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DropdownSectionProps {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DropdownItem {
    pub props: DropdownItemProps,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DropdownItemProps {
    pub title: String,
    pub value: String,
}
