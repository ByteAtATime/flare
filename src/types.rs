use crate::components::Component;
use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct ToastOptions {
    pub title: String,
    pub message: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tree {
    pub id: String,
    pub children: Vec<Component>,
}
