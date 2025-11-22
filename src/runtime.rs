use crate::globals::SENDER;
use crate::message::Message;
use crate::types::{SidecarRequest, SidecarResponse, Tree};
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
                        if let Err(e) = sender.send(Message::UpdateToast(title)).await {
                            eprintln!("Failed to send toast message: {:?}", e);
                        }
                        let response = SidecarRequest::Response {
                            id,
                            result: Some(Value::Null),
                            error: None,
                        };
                        let mut stdin = stdin_clone.lock().unwrap();
                        if let Ok(data) = rmp_serde::to_vec(&response) {
                            let length = (data.len() as u32).to_be_bytes();
                            let _ = stdin.write_all(&length);
                            let _ = stdin.write_all(&data);
                            let _ = stdin.flush();
                        }
                    });
                });
            }
        }
        SidecarResponse::UpdateTree { id, tree } => {
            if let Ok(tree) = serde_json::from_value::<Tree>(tree) {
                if let Some(mut sender) = SENDER.lock().unwrap().clone() {
                    let stdin_clone = stdin.clone();
                    thread::spawn(move || {
                        iced::futures::executor::block_on(async move {
                            if let Err(e) = sender.send(Message::UpdateTree(tree)).await {
                                eprintln!("Failed to send tree update: {:?}", e);
                            }
                            let response = SidecarRequest::Response {
                                id,
                                result: Some(Value::Null),
                                error: None,
                            };
                            let mut stdin = stdin_clone.lock().unwrap();
                            if let Ok(data) = rmp_serde::to_vec(&response) {
                                let length = (data.len() as u32).to_be_bytes();
                                let _ = stdin.write_all(&length);
                                let _ = stdin.write_all(&data);
                                let _ = stdin.flush();
                            }
                        });
                    });
                }
            }
        }
        SidecarResponse::CacheSet { id, .. } => {
            let response = SidecarRequest::Response {
                id,
                result: Some(Value::Null),
                error: None,
            };
            let data = rmp_serde::to_vec(&response)?;
            let length = (data.len() as u32).to_be_bytes();

            let mut stdin = stdin.lock().unwrap();
            stdin.write_all(&length)?;
            stdin.write_all(&data)?;
            stdin.flush()?;
        }
    }

    Ok(())
}

pub fn setup_and_run(mut callback_receiver: mpsc::UnboundedReceiver<(String, Value)>) {
    let plugin_path = std::env::current_dir()
        .unwrap()
        .join("test/plugin.js")
        .to_string_lossy()
        .to_string();

    let mut runtime = match SidecarRuntime::new(&plugin_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to start sidecar: {:?}", e);
            return;
        }
    };

    loop {
        let msg = iced::futures::executor::block_on(callback_receiver.next());

        match msg {
            Some((callback_id, args)) => {
                let request = SidecarRequest::InvokeCallback { callback_id, args };
                if let Err(e) = runtime.send_request(&request) {
                    eprintln!("Failed to send callback request: {:?}", e);
                }
            }
            None => break,
        }
    }
}
