use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use serde_json::Value;
use std::process::{Child, Command, Stdio};
use std::thread;

use crate::globals;
use crate::handlers;
use crate::transport::{MessageReader, Transport};
use crate::types::{RustResponse, SidecarRequest, SidecarResponse};

pub struct SidecarRuntime {
    process: Child,
    transport: Transport,
}

impl SidecarRuntime {
    pub fn new(plugin_path: &str) -> Result<Self, std::io::Error> {
        let sidecar_path = std::env::current_exe()?.parent().unwrap().join("sidecar");

        let mut process = Command::new(sidecar_path)
            .arg(plugin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        let transport = Transport::new(stdin);
        let mut reader = MessageReader::new(stdout);

        let transport_clone = transport.clone();
        thread::spawn(move || {
            loop {
                match reader.read_next() {
                    Some(data) => {
                        if let Err(e) = handle_sidecar_response(&data, &transport_clone) {
                            eprintln!("Failed to handle sidecar response: {:?}", e);
                        }
                    }
                    None => break,
                }
            }
        });

        Ok(Self { process, transport })
    }

    pub fn send_request(&mut self, request: &SidecarRequest) -> Result<(), std::io::Error> {
        self.transport.send_request(request)
    }
}

impl Drop for SidecarRuntime {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

fn handle_sidecar_response(
    data: &[u8],
    transport: &Transport,
) -> Result<(), Box<dyn std::error::Error>> {
    let response: SidecarResponse = rmp_serde::from_slice(data)?;

    let (id, result) = match response {
        SidecarResponse::Initialized { success, error } => {
            if !success {
                eprintln!("Plugin initialization failed: {:?}", error);
            }
            return Ok(());
        }
        SidecarResponse::CallbackResult { success, error } => {
            if !success {
                eprintln!("Callback failed: {:?}", error);
            }
            return Ok(());
        }

        SidecarResponse::ShowToast { id, title, .. } => (id, handlers::ui::show_toast(title)),
        SidecarResponse::UpdateTree { id, tree } => (id, handlers::ui::update_tree(tree)),
        SidecarResponse::Pop { id } => (id, handlers::ui::pop()),
        SidecarResponse::OpenExtensionPreferences { id }
        | SidecarResponse::OpenCommandPreferences { id } => (id, handlers::ui::open_settings()),
        SidecarResponse::OpenUrl { id, url } => (id, handlers::ui::open_url(url)),

        SidecarResponse::LocalStorageSet {
            id,
            namespace,
            key,
            data,
        } => (id, handlers::storage::set(namespace, key, data)),
        SidecarResponse::LocalStorageGet { id, namespace, key } => {
            (id, handlers::storage::get(namespace, key))
        }
        SidecarResponse::LocalStorageRemove { id, namespace, key } => {
            (id, handlers::storage::remove(namespace, key))
        }
        SidecarResponse::LocalStorageClear { id, namespace } => {
            (id, handlers::storage::clear(namespace))
        }
        SidecarResponse::LocalStorageAll { id, namespace } => {
            (id, handlers::storage::get_all(namespace))
        }

        SidecarResponse::ClipboardCopy {
            id,
            content,
            concealed,
        } => (id, handlers::clipboard::copy(content, concealed)),
        SidecarResponse::ClipboardClear { id } => (id, handlers::clipboard::clear()),
        SidecarResponse::ClipboardRead { id, .. } => (id, handlers::clipboard::read()),

        SidecarResponse::OAuthAuthorize { id, url, state } => {
            handlers::oauth::authorize(id, url, state, transport);
            return Ok(());
        }
        SidecarResponse::OAuthSetTokens {
            id,
            provider_id,
            tokens,
        } => (id, handlers::oauth::set_tokens(provider_id, tokens)),
        SidecarResponse::OAuthGetTokens { id, provider_id } => {
            (id, handlers::oauth::get_tokens(provider_id))
        }
        SidecarResponse::OAuthRemoveTokens { id, provider_id } => {
            (id, handlers::oauth::remove_tokens(provider_id))
        }
    };

    let rust_response = match result {
        Ok(res) => RustResponse::Success { id, result: res },
        Err(e) => RustResponse::Error { id, error: e },
    };

    transport.send(&rust_response)?;
    Ok(())
}

pub fn launch_extension(
    plugin_path: &str,
    assets_path: &str,
    preferences: std::collections::HashMap<String, Value>,
) -> Result<(), std::io::Error> {
    let mut runtime_guard = globals::RUNTIME.lock().unwrap();
    if runtime_guard.is_some() {
        drop(runtime_guard.take());
    }

    let mut runtime = SidecarRuntime::new(plugin_path)?;
    runtime.send_request(&SidecarRequest::Initialize {
        preferences,
        assets_path: assets_path.to_string(),
    })?;
    *runtime_guard = Some(runtime);
    Ok(())
}

pub fn stop_runtime() {
    let mut runtime_guard = globals::RUNTIME.lock().unwrap();
    drop(runtime_guard.take());
}

pub fn run_callback_loop(mut callback_receiver: mpsc::UnboundedReceiver<(String, Value)>) {
    loop {
        let msg = iced::futures::executor::block_on(callback_receiver.next());

        match msg {
            Some((callback_id, args)) => {
                let request = SidecarRequest::InvokeCallback { callback_id, args };
                if let Some(runtime) = globals::RUNTIME.lock().unwrap().as_mut() {
                    if let Err(e) = runtime.send_request(&request) {
                        eprintln!("Failed to send callback request: {:?}", e);
                    }
                }
            }
            None => break,
        }
    }
}
