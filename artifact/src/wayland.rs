use anyhow::Result;
use std::process::Command;
use crate::SharedState;

pub struct ArtifactRenderer {
    state: SharedState,
}

impl ArtifactRenderer {
    pub fn new(state: SharedState) -> Result<Self> {
        Ok(Self { state })
    }

    pub async fn run(&self) -> Result<()> {
        let mut child = None;

        loop {
            let should_run = self.state.is_running();

            if should_run && child.is_none() {
                // Spawn Python artifact.py renderer
                child = Some(Command::new("python3")
                    .arg("/home/abyss/dead/artifact.py")
                    .spawn()?);
            } else if !should_run && child.is_some() {
                // Kill the process
                if let Some(mut c) = child.take() {
                    let _ = c.kill();
                    let _ = c.wait();
                }
            }

            // Update corruption level file for Python to read
            let corruption_level = self.state.get_corruption_level();
            let _ = std::fs::write("/tmp/artifact.level", corruption_level.to_string());

            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        }
    }
}
