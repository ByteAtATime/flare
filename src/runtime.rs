use crate::transport::{SidecarReader, SidecarWriter};
use crate::types::SidecarRequest;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SidecarListener {
    pub reader: Arc<Mutex<SidecarReader>>,
}

impl Hash for SidecarListener {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.reader).hash(state);
    }
}

pub async fn launch_extension(
    plugin_path: &str,
    assets_path: &str,
    preferences: HashMap<String, Value>,
) -> Result<(SidecarWriter, SidecarReader), std::io::Error> {
    let sidecar_path = std::env::current_exe()?.parent().unwrap().join("sidecar");

    let mut process = Command::new(sidecar_path)
        .arg(plugin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = process.stdin.take().unwrap();
    let stdout = process.stdout.take().unwrap();

    let writer = SidecarWriter::new(stdin);
    let reader = SidecarReader::new(stdout);

    writer
        .send_request(&SidecarRequest::Initialize {
            preferences,
            assets_path: assets_path.to_string(),
        })
        .await?;

    Ok((writer, reader))
}
