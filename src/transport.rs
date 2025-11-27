use std::io::{Read, Write};
use std::process::{ChildStdin, ChildStdout};
use std::sync::{Arc, Mutex};

use crate::types::{RustResponse, SidecarRequest};

#[derive(Clone)]
pub struct Transport {
    stdin: Arc<Mutex<ChildStdin>>,
}

impl Transport {
    pub fn new(stdin: ChildStdin) -> Self {
        Self {
            stdin: Arc::new(Mutex::new(stdin)),
        }
    }

    pub fn send(&self, response: &RustResponse) -> Result<(), Box<dyn std::error::Error>> {
        let data = rmp_serde::to_vec_named(response)?;
        let length = (data.len() as u32).to_be_bytes();

        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(&length)?;
        stdin.write_all(&data)?;
        stdin.flush()?;
        Ok(())
    }

    pub fn send_request(&self, request: &SidecarRequest) -> Result<(), std::io::Error> {
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

pub struct MessageReader {
    stdout: ChildStdout,
    buffer: Vec<u8>,
    length_buf: [u8; 4],
}

impl MessageReader {
    pub fn new(stdout: ChildStdout) -> Self {
        Self {
            stdout,
            buffer: Vec::new(),
            length_buf: [0; 4],
        }
    }

    pub fn read_next(&mut self) -> Option<Vec<u8>> {
        if self.stdout.read_exact(&mut self.length_buf).is_err() {
            return None;
        }

        let length = u32::from_be_bytes(self.length_buf) as usize;
        self.buffer.resize(length, 0);

        if self.stdout.read_exact(&mut self.buffer).is_err() {
            return None;
        }

        Some(self.buffer.clone())
    }
}
