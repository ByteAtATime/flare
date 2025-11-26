use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

const IPC_COMMAND_TOGGLE: &[u8] = b"toggle";
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

pub fn is_daemon_running() -> bool {
    let path = socket_path();
    UnixStream::connect(&path).is_ok()
}

pub fn start_listener<F>(on_toggle: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() + Send + 'static,
{
    let path = socket_path();

    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 64];
                if let Ok(n) = stream.read(&mut buf) {
                    if &buf[..n] == IPC_COMMAND_TOGGLE {
                        on_toggle();
                        let _ = stream.write_all(IPC_RESPONSE_OK);
                    }
                }
            }
        }
    });

    Ok(())
}

pub fn cleanup() {
    let path = socket_path();
    let _ = std::fs::remove_file(path);
}
