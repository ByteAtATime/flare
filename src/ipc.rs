use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::{Subscription, stream};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

const IPC_COMMAND_TOGGLE: &[u8] = b"toggle";
const IPC_COMMAND_OAUTH_PREFIX: &[u8] = b"oauth:";
const IPC_RESPONSE_OK: &[u8] = b"ok";

fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("flare.sock")
}

pub fn send_toggle() -> Result<(), Box<dyn std::error::Error>> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)?;
    stream.write_all(IPC_COMMAND_TOGGLE)?;

    let mut response = [0u8; 16];
    let n = stream.read(&mut response)?;
    if &response[..n] == IPC_RESPONSE_OK {
        Ok(())
    } else {
        Err("Unexpected response from daemon".into())
    }
}

pub fn send_oauth_redirect(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)?;

    let mut msg = Vec::from(IPC_COMMAND_OAUTH_PREFIX);
    msg.extend_from_slice(url.as_bytes());
    stream.write_all(&msg)?;

    let mut response = [0u8; 16];
    let n = stream.read(&mut response)?;
    if &response[..n] == IPC_RESPONSE_OK {
        Ok(())
    } else {
        Err("Unexpected response from daemon".into())
    }
}

pub fn is_daemon_running() -> bool {
    let path = socket_path();
    println!("Checking daemon socket at: {:?}", path);
    UnixStream::connect(&path).is_ok()
}

fn listener() -> impl Stream<Item = crate::Message> {
    stream::channel(100, |mut output: mpsc::Sender<crate::Message>| async move {
        let path = socket_path();

        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind IPC socket: {}", e);
                std::future::pending::<()>().await;
                return;
            }
        };

        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 2048];
                if let Ok(n) = stream.read(&mut buf).await {
                    let data = &buf[..n];
                    if data == IPC_COMMAND_TOGGLE {
                        let _ = output.send(crate::Message::ToggleWindow).await;
                        let _ = stream.write_all(IPC_RESPONSE_OK).await;
                    } else if data.starts_with(IPC_COMMAND_OAUTH_PREFIX) {
                        if let Ok(url) =
                            std::str::from_utf8(&data[IPC_COMMAND_OAUTH_PREFIX.len()..])
                        {
                            let _ = output
                                .send(crate::Message::HandleOAuthRedirect(url.to_string()))
                                .await;
                            let _ = stream.write_all(IPC_RESPONSE_OK).await;
                        }
                    }
                }
            }
        }
    })
}

pub fn subscription() -> Subscription<crate::Message> {
    Subscription::run(listener)
}

pub fn cleanup() {
    let path = socket_path();
    let _ = std::fs::remove_file(path);
}
