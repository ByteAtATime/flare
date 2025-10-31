use rustyscript::serde_json::Value;

#[derive(serde::Deserialize)]
pub struct ToastOptions {
    pub title: String,
    pub message: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Tree {
    pub id: String,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TreeNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub props: Option<Value>,
    pub children: Vec<TreeNode>,
}
