use crate::types::SidecarRequest;
use iced::futures::channel::mpsc;
use iced::futures::StreamExt;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

pub struct SidecarRuntime {
    process: Child,
    stdin: std::process::ChildStdin,
    stdout_reader: BufReader<std::process::ChildStdout>,
}

impl SidecarRuntime {
    pub fn new(plugin_path: &str) -> Result<Self, std::io::Error> {
        let sidecar_path = std::env::current_exe()?
            .parent()
            .unwrap()
            .join("sidecar");

        let mut process = Command::new(sidecar_path)
            .arg(plugin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let stdout_reader = BufReader::new(stdout);

        Ok(Self {
            process,
            stdin,
            stdout_reader,
        })
    }

    pub fn send_request(&mut self, request: &SidecarRequest) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(request)?;
        writeln!(self.stdin, "{}", json)?;
        self.stdin.flush()?;
        Ok(())
    }

    pub fn read_line(&mut self) -> Result<String, std::io::Error> {
        let mut line = String::new();
        self.stdout_reader.read_line(&mut line)?;
        Ok(line)
    }
}

impl Drop for SidecarRuntime {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
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

    if let Err(e) = runtime.read_line() {
        eprintln!("Failed to read initialization response: {:?}", e);
        return;
    }

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
