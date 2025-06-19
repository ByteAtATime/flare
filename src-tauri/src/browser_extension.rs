use futures_util::{stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum IncomingMessage {
    Request {
        id: u64,
        method: String,
        params: Option<serde_json::Value>,
    },
    Response {
        id: u64,
        #[serde(default)]
        result: serde_json::Value,
        #[serde(default)]
        error: serde_json::Value,
    },
    Notification {
        method: String,
        params: Option<serde_json::Value>,
    },
}

pub struct WsState {
    pub connection: Arc<Mutex<Option<tokio::sync::mpsc::Sender<String>>>>,
    pub is_connected: Arc<Mutex<bool>>,
    pub pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>>,
    pub request_id_counter: Arc<Mutex<u64>>,
}

impl Default for WsState {
    fn default() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            request_id_counter: Arc::new(Mutex::new(0)),
        }
    }
}

async fn handle_connection(stream: TcpStream, app_handle: AppHandle) {
    let state: State<WsState> = app_handle.state();
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake error: {}", e);
            return;
        }
    };

    *state.is_connected.lock().unwrap() = true;
    println!("Browser extension connected.");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    *state.connection.lock().unwrap() = Some(tx);

    let sender_task = tokio::spawn(async move {
        while let Some(msg_to_send) = rx.recv().await {
            if ws_sender.send(Message::Binary(msg_to_send.into())).await.is_err() {
                break;
            }
        }
    });

    let app_clone_for_receiver = app_handle.clone();
    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(_) => break,
            };

            if let Message::Text(text) = msg {
                match serde_json::from_str::<IncomingMessage>(&text) {
                    Ok(IncomingMessage::Request { id, method, .. }) => {
                        if method == "ping" {
                            let response = json!({
                                "jsonrpc": "2.0",
                                "result": null,
                                "id": id
                            });
                            let tx = app_clone_for_receiver.state::<WsState>().connection.lock().unwrap().clone();
                            if let Some(tx) = tx {
                                let _ = tx.send(response.to_string()).await;
                            }
                        }
                    }
                    Ok(IncomingMessage::Response { id, result, error }) => {
                        let sender = app_clone_for_receiver.state::<WsState>().pending_requests.lock().unwrap().remove(&id);
                        if let Some(sender) = sender {
                            if !error.is_null() {
                                let _ = sender.send(Err(error.to_string()));
                            } else {
                                let _ = sender.send(Ok(result));
                            }
                        }
                    }
                    Ok(IncomingMessage::Notification { method, params }) => {
                        println!("Received notification: {} with params {:?}", method, params);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse message from browser extension: {}", e);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }

    *state.is_connected.lock().unwrap() = false;
    *state.connection.lock().unwrap() = None;
    println!("Browser extension disconnected.");
}

pub async fn run_server(app_handle: AppHandle) {
    let addr = "127.0.0.1:7265";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, app_handle.clone()));
    }
}