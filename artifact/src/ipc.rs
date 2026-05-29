use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::fs;

use crate::SharedState;

#[derive(Serialize, Deserialize, Debug)]
pub struct IpcMessage {
    pub command: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IpcResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn run(&self, state: SharedState) -> Result<()> {
        // Remove existing socket file
        let _ = fs::remove_file(&self.socket_path).await;

        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    }
}

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn send_command(&self, command: &str) -> Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        let msg = IpcMessage {
            command: command.to_string(),
            args: serde_json::json!({}),
        };

        let request = serde_json::to_string(&msg)? + "\n";
        stream.write_all(request.as_bytes()).await?;

        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let parsed: IpcResponse = serde_json::from_str(&response)?;

        if let Some(data) = parsed.data {
            Ok(format!("{}\n{}", parsed.message, serde_json::to_string_pretty(&data)?))
        } else {
            Ok(parsed.message)
        }
    }

    pub async fn send_command_with_arg(&self, command: &str, value: u64) -> Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        let msg = IpcMessage {
            command: command.to_string(),
            args: serde_json::json!({"level": value}),
        };

        let request = serde_json::to_string(&msg)? + "\n";
        stream.write_all(request.as_bytes()).await?;

        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let parsed: IpcResponse = serde_json::from_str(&response)?;

        if let Some(data) = parsed.data {
            Ok(format!("{}\n{}", parsed.message, serde_json::to_string_pretty(&data)?))
        } else {
            Ok(parsed.message)
        }
    }
}

async fn handle_connection(stream: UnixStream, state: SharedState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let msg: IpcMessage = serde_json::from_str(&line)?;
        let response = process_command(&msg, &state).await;

        writer.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}

async fn process_command(msg: &IpcMessage, state: &SharedState) -> IpcResponse {
    match msg.command.as_str() {
        "start" => {
            state.set_running(true);
            IpcResponse {
                success: true,
                message: "Artifact simulation started".to_string(),
                data: None,
            }
        }
        "stop" => {
            state.set_running(false);
            IpcResponse {
                success: true,
                message: "Artifact simulation stopped".to_string(),
                data: None,
            }
        }
        "status" => {
            let is_running = state.is_running();
            let corruption = state.get_corruption_level();
            let stage = corruption / 20;
            IpcResponse {
                success: true,
                message: format!("{}% (stage {}) {}", corruption, stage, if is_running { "[running]" } else { "[stopped]" }),
                data: None,
            }
        }
        "change" => {
            if let Some(level) = msg.args.get("level").and_then(|v| v.as_u64()) {
                if level >= 1 && level <= 100 {
                    state.set_corruption_level(level);
                    // Update manual change timestamp
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    state.manual_change_time.store(now, std::sync::atomic::Ordering::Relaxed);
                    IpcResponse {
                        success: true,
                        message: format!("Corruption level set to {}%", level),
                        data: None,
                    }
                } else {
                    IpcResponse {
                        success: false,
                        message: "Level must be between 1 and 100".to_string(),
                        data: None,
                    }
                }
            } else {
                IpcResponse {
                    success: false,
                    message: "Invalid level argument".to_string(),
                    data: None,
                }
            }
        }
        _ => IpcResponse {
            success: false,
            message: format!("Unknown command: {}", msg.command),
            data: None,
        },
    }
}
