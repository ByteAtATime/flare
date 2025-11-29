use crate::transport::SidecarWriter;
use crate::types::RustResponse;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static OAUTH_PENDING: LazyLock<Mutex<HashMap<String, (u32, SidecarWriter)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const OAUTH_NAMESPACE: &str = "__flare_oauth_tokens__";

pub fn authorize(id: u32, url: String, state: String, writer: SidecarWriter) {
    OAUTH_PENDING.lock().unwrap().insert(state, (id, writer));

    let _ = crate::utils::open_url(&url);
}

pub fn complete(state: &str, code: &str) -> bool {
    let entry = OAUTH_PENDING.lock().unwrap().remove(state);

    if let Some((id, writer)) = entry {
        let code = code.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let response = RustResponse::Success {
                    id,
                    result: Some(serde_json::json!({ "authorizationCode": code })),
                };
                let _ = writer.send(&response).await;
            });
        });

        true
    } else {
        false
    }
}

pub fn set_tokens(
    provider_id: String,
    tokens: String,
) -> Result<Option<serde_json::Value>, String> {
    crate::storage::set(OAUTH_NAMESPACE, &provider_id, &tokens).map(|_| None)
}

pub fn get_tokens(provider_id: String) -> Result<Option<serde_json::Value>, String> {
    Ok(crate::storage::get(OAUTH_NAMESPACE, &provider_id).map(serde_json::Value::String))
}

pub fn remove_tokens(provider_id: String) -> Result<Option<serde_json::Value>, String> {
    crate::storage::remove(OAUTH_NAMESPACE, &provider_id);
    Ok(None)
}
