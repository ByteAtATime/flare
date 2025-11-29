use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;

use crate::types::{RustResponse, SidecarRequest, SidecarResponse};

#[derive(Clone, Debug)]
pub struct SidecarWriter {
    stdin: Arc<Mutex<ChildStdin>>,
}

impl SidecarWriter {
    pub fn new(stdin: ChildStdin) -> Self {
        Self {
            stdin: Arc::new(Mutex::new(stdin)),
        }
    }

    pub async fn send(&self, response: &RustResponse) -> Result<(), std::io::Error> {
        let data = rmp_serde::to_vec_named(response)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.write_bytes(data).await
    }

    pub async fn send_request(&self, request: &SidecarRequest) -> Result<(), std::io::Error> {
        let data = rmp_serde::to_vec_named(request)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.write_bytes(data).await
    }

    async fn write_bytes(&self, data: Vec<u8>) -> Result<(), std::io::Error> {
        let length = (data.len() as u32).to_be_bytes();
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(&length).await?;
        stdin.write_all(&data).await?;
        stdin.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SidecarReader {
    stdout: ChildStdout,
    buffer: Vec<u8>,
    length_buf: [u8; 4],
}

impl SidecarReader {
    pub fn new(stdout: ChildStdout) -> Self {
        Self {
            stdout,
            buffer: Vec::new(),
            length_buf: [0; 4],
        }
    }

    pub async fn read_next(&mut self) -> Option<SidecarResponse> {
        use tokio::io::AsyncReadExt;

        let mut length_buf = [0u8; 4];
        if self.stdout.read_exact(&mut length_buf).await.is_err() {
            return None;
        }
        let length = u32::from_be_bytes(length_buf) as usize;

        let mut buffer = vec![0u8; length];
        if self.stdout.read_exact(&mut buffer).await.is_err() {
            return None;
        }

        rmp_serde::from_slice(&buffer).ok()
    }
}
