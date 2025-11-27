use crate::globals::SENDER;
use crate::message::Message;
use crate::types::{RustResponse, SidecarRequest, SidecarResponse};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct SidecarRuntime {
    process: Child,
    stdin: Arc<Mutex<std::process::ChildStdin>>,
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

        let stdin = Arc::new(Mutex::new(process.stdin.take().unwrap()));
        let mut stdout = process.stdout.take().unwrap();

        let stdin_clone = stdin.clone();
        thread::spawn(move || {
            let mut length_buf = [0u8; 4];
            loop {
                if stdout.read_exact(&mut length_buf).is_err() {
                    break;
                }
                let length = u32::from_be_bytes(length_buf) as usize;

                let mut message_buf = vec![0u8; length];
                if stdout.read_exact(&mut message_buf).is_err() {
                    break;
                }

                if let Err(e) = handle_sidecar_response(&message_buf, &stdin_clone) {
                    eprintln!("Failed to handle sidecar response: {:?}", e);
                    fn to_hex_string(bytes: &[u8]) -> String {
                        bytes.iter().map(|b| format!("{:02x}", b)).collect()
                    }
                    eprintln!("{:?}", to_hex_string(&message_buf));
                }
            }
        });

        Ok(Self { process, stdin })
    }

    pub fn send_request(&mut self, request: &SidecarRequest) -> Result<(), std::io::Error> {
        let data = rmp_serde::to_vec_named(request)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let length = (data.len() as u32).to_be_bytes();

        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(&length)?;
        stdin.write_all(&data)?;
        stdin.flush()?;
        Ok(())
    }
}

impl Drop for SidecarRuntime {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

fn handle_sidecar_response(
    data: &[u8],
    stdin: &Arc<Mutex<std::process::ChildStdin>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response: SidecarResponse = rmp_serde::from_slice(data)?;

    match response {
        SidecarResponse::Initialized { success, error } => {
            if !success {
                eprintln!("Plugin initialization failed: {:?}", error);
            }
        }
        SidecarResponse::CallbackResult { success, error } => {
            if !success {
                eprintln!("Callback failed: {:?}", error);
            }
        }
        SidecarResponse::ShowToast {
            id,
            title,
            message: _,
            style: _,
        } => {
            if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                let stdin_clone = stdin.clone();
                thread::spawn(move || {
                    iced::futures::executor::block_on(async move {
                        if let Err(e) = sender.send(Message::ShowToast(title)).await {
                            eprintln!("Failed to send toast message: {:?}", e);
                        }
                        let response = RustResponse::Success { id, result: None };
                        let _ = send_response(&response, &stdin_clone);
                    });
                });
            }
        }
        SidecarResponse::UpdateTree { id, tree } => {
            if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                let stdin_clone = stdin.clone();
                thread::spawn(move || {
                    iced::futures::executor::block_on(async move {
                        if let Err(e) = sender.send(Message::UpdateTree(tree)).await {
                            eprintln!("Failed to send tree update: {:?}", e);
                        }
                        let response = RustResponse::Success { id, result: None };
                        let _ = send_response(&response, &stdin_clone);
                    });
                });
            }
        }
        SidecarResponse::CacheSet {
            id,
            namespace,
            key,
            data,
        } => {
            let result = match crate::cache::set(&namespace, &key, &data) {
                Ok(_) => RustResponse::Success { id, result: None },
                Err(e) => RustResponse::Error { id, error: e },
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::CacheGet { id, namespace, key } => {
            let result = match crate::cache::get(&namespace, &key) {
                Some(data) => RustResponse::Success {
                    id,
                    result: Some(Value::String(data)),
                },
                None => RustResponse::Success {
                    id,
                    result: Some(Value::Null),
                },
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::CacheHas { id, namespace, key } => {
            let has = crate::cache::has(&namespace, &key);
            let result = RustResponse::Success {
                id,
                result: Some(Value::Bool(has)),
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::CacheRemove { id, namespace, key } => {
            let removed = crate::cache::remove(&namespace, &key);
            let result = RustResponse::Success {
                id,
                result: Some(Value::Bool(removed)),
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::CacheClear { id, namespace } => {
            let result = match crate::cache::clear(&namespace) {
                Ok(_) => RustResponse::Success { id, result: None },
                Err(e) => RustResponse::Error { id, error: e },
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::CacheIsEmpty { id, namespace } => {
            let is_empty = crate::cache::is_empty(&namespace);
            let result = RustResponse::Success {
                id,
                result: Some(Value::Bool(is_empty)),
            };
            send_response(&result, stdin)?;
        }
        SidecarResponse::Pop { id } => {
            if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                let stdin_clone = stdin.clone();
                thread::spawn(move || {
                    iced::futures::executor::block_on(async move {
                        if let Err(e) = sender.send(Message::PopToRoot).await {
                            eprintln!("Failed to send PopToRoot message: {:?}", e);
                        }
                        let response = RustResponse::Success { id, result: None };
                        let _ = send_response(&response, &stdin_clone);
                    });
                });
            }
        }
    }

    Ok(())
}

fn send_response(
    response: &RustResponse,
    stdin: &Arc<Mutex<std::process::ChildStdin>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = rmp_serde::to_vec_named(response)?;
    let length = (data.len() as u32).to_be_bytes();

    let mut stdin = stdin.lock().unwrap();
    stdin.write_all(&length)?;
    stdin.write_all(&data)?;
    stdin.flush()?;
    Ok(())
}

pub fn launch_extension(
    plugin_path: &str,
    preferences: std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), std::io::Error> {
    let mut runtime_guard = crate::globals::RUNTIME.lock().unwrap();
    if runtime_guard.is_some() {
        drop(runtime_guard.take());
    }

    let mut runtime = SidecarRuntime::new(plugin_path)?;
    runtime.send_request(&SidecarRequest::Initialize { preferences })?;
    *runtime_guard = Some(runtime);
    Ok(())
}

pub fn stop_runtime() {
    let mut runtime_guard = crate::globals::RUNTIME.lock().unwrap();
    drop(runtime_guard.take());
}

pub fn run_callback_loop(mut callback_receiver: mpsc::UnboundedReceiver<(String, Value)>) {
    loop {
        let msg = iced::futures::executor::block_on(callback_receiver.next());

        match msg {
            Some((callback_id, args)) => {
                let request = SidecarRequest::InvokeCallback { callback_id, args };
                if let Some(runtime) = crate::globals::RUNTIME.lock().unwrap().as_mut() {
                    if let Err(e) = runtime.send_request(&request) {
                        eprintln!("Failed to send callback request: {:?}", e);
                    }
                }
            }
            None => break,
        }
    }
}
