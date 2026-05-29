use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::env;

mod wayland;
mod ipc;
mod corruption;
mod progress;

use ipc::{IpcServer, IpcClient};
use progress::ProgressTracker;
use wayland::ArtifactRenderer;

#[derive(Clone)]
pub struct SharedState {
    running: Arc<AtomicBool>,
    corruption_level: Arc<AtomicU64>,
    manual_change_time: Arc<AtomicU64>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            corruption_level: Arc::new(AtomicU64::new(0)),
            manual_change_time: Arc::new(AtomicU64::new(0)),
        }
    }

    fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Relaxed);
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn set_corruption_level(&self, level: u64) {
        self.corruption_level.store(level.min(100), Ordering::Relaxed);
    }

    fn get_corruption_level(&self) -> u64 {
        self.corruption_level.load(Ordering::Relaxed)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let socket_path = PathBuf::from("/tmp/artifact.sock");

    // Parse command
    match args.get(1).map(|s| s.as_str()) {
        Some("start") => {
            let client = IpcClient::new(socket_path);
            let resp = client.send_command("start").await?;
            println!("{}", resp);
            return Ok(());
        }
        Some("stop") => {
            let client = IpcClient::new(socket_path);
            let resp = client.send_command("stop").await?;
            println!("{}", resp);
            return Ok(());
        }
        Some("status") => {
            let client = IpcClient::new(socket_path);
            let resp = client.send_command("status").await?;
            println!("{}", resp);
            return Ok(());
        }
        Some("change") => {
            if let Some(level_str) = args.get(2) {
                if let Ok(level) = level_str.parse::<u64>() {
                    if level >= 1 && level <= 100 {
                        let client = IpcClient::new(socket_path);
                        let resp = client.send_command_with_arg("change", level).await?;
                        println!("{}", resp);
                        return Ok(());
                    } else {
                        println!("Error: level must be between 1 and 100");
                        return Ok(());
                    }
                } else {
                    println!("Error: invalid number");
                    return Ok(());
                }
            } else {
                println!("Usage: artifact change <1-100>");
                return Ok(());
            }
        }
        None => {} // Run as daemon
        Some(cmd) => {
            println!("artifact — GPU artifact simulator");
            println!();
            println!("Commands:");
            println!("  artifact start          Start the corruption progression");
            println!("  artifact stop           Stop the simulation");
            println!("  artifact status         Show corruption level (0-100%)");
            println!("  artifact change <N>     Set corruption to N% (1-100)");
            return Ok(());
        }
    }

    // Run as daemon
    let state = SharedState::new();

    // Spawn IPC server
    let ipc_state = state.clone();
    let ipc_path = socket_path.clone();
    tokio::spawn(async move {
        let _ = IpcServer::new(ipc_path).run(ipc_state).await;
    });

    // Spawn progress tracker
    let progress_state = state.clone();
    tokio::spawn(async move {
        let mut tracker = ProgressTracker::new(std::time::Duration::from_secs(5 * 60));
        let mut was_running = false;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        progress_state.manual_change_time.store(now, Ordering::Relaxed);

        loop {
            let is_running = progress_state.is_running();
            if is_running != was_running {
                tracker.set_running(is_running);
                was_running = is_running;
            }

            // Only update from tracker if no recent manual change
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let last_manual = progress_state.manual_change_time.load(Ordering::Relaxed);

            if is_running && (now - last_manual) > 300 {
                let percent = tracker.get_progress_percent();
                progress_state.set_corruption_level(percent);
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // Run Wayland renderer
    let renderer_state = state.clone();
    let renderer = ArtifactRenderer::new(renderer_state)?;
    renderer.run().await?;

    Ok(())
}
