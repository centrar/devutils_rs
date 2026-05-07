//! Sandbox Module - Isolated command execution via Docker
//! Ensures the autonomous agent cannot damage the host system.

use std::process::Command;
use std::path::Path;

pub struct Sandbox {
    image: String,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            image: "rust:latest".to_string(), // Default image
        }
    }

    /// Execute a command in a Docker container
    pub fn execute(&self, command: &str, workspace: &Path) -> Result<String, String> {
        // Mitigation for LOW-5: Block obvious destructive commands
        let blocklist = ["rm -rf /", "rm -rf *", "mkfs"];
        for blocked in blocklist {
            if command.contains(blocked) {
                return Err(format!("Command blocked by sandbox security policy: {}", blocked));
            }
        }

        let workspace_str = workspace.to_string_lossy();
        
        // docker run --rm -v "C:\path\to\ws:/workspace" -w /workspace rust:latest sh -c "command"
        let output = Command::new("docker")
            .args([
                "run", "--rm",
                "-v", &format!("{}:/workspace", workspace_str),
                "-w", "/workspace",
                &self.image,
                "sh", "-c", command
            ])
            .output()
            .map_err(|e| {
                // Ensure the error message mentions Docker so the test can detect missing Docker.
                format!("Docker error: {}", e)
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Set the Docker image to use for sandboxing
    pub fn set_image(&mut self, image: &str) {
        self.image = image.to_string();
    }
}
