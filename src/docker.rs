//! Docker Sessions - Isolated Agent Environments
//!
//! Run agents in isolated Docker containers similar to OpenCode
//! - Create persistent workspace containers
//! - Clean isolation for dangerous operations
//! - Resource limits and monitoring

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::RwLock;

static DOCKER_SESSIONS: Lazy<RwLock<HashMap<String, DockerSession>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerSession {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: SessionStatus,
    pub created_at: u64,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Creating,
    Running,
    Paused,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image: String,
    pub memory_limit: Option<String>,
    pub cpu_limit: Option<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: HashMap<String, String>,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            memory_limit: Some("512m".to_string()),
            cpu_limit: Some("0.5".to_string()),
            ports: vec![],
            volumes: vec![],
            environment: HashMap::new(),
        }
    }
}

pub fn create_session(name: &str, config: &DockerConfig) -> Result<String> {
    let id = format!("devutils_{}", name);

    // Build docker run command
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        id.clone(),
        "--hostname".to_string(),
        name.to_string(),
    ];

    // Add memory limit
    if let Some(mem) = &config.memory_limit {
        args.push("--memory".to_string());
        args.push(mem.clone());
    }

    // Add CPU limit
    if let Some(cpu) = &config.cpu_limit {
        args.push("--cpus".to_string());
        args.push(cpu.clone());
    }

    // Add ports
    for port in &config.ports {
        args.push("-p".to_string());
        args.push(port.clone());
    }

    // Add volumes
    for vol in &config.volumes {
        args.push("-v".to_string());
        args.push(vol.clone());
    }

    // Add environment
    for (key, val) in &config.environment {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, val));
    }

    // Add workspace mount
    args.push("-v".to_string());
    args.push(".:/workspace".to_string());

    // Keep container alive
    args.push("tail".to_string());
    args.push("-f".to_string());
    args.push("/dev/null".to_string());

    args.push(config.image.clone());

    let output = Command::new("docker").args(&args).output()?;

    if output.status.success() {
        let session = DockerSession {
            id: id.clone(),
            name: name.to_string(),
            image: config.image.clone(),
            status: SessionStatus::Running,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ports: config.ports.clone(),
            volumes: config.volumes.clone(),
            environment: config.environment.clone(),
        };

        let mut sessions = DOCKER_SESSIONS.write().unwrap();
        sessions.insert(id.clone(), session);

        Ok(format!("Created Docker session: {}", id))
    } else {
        Err(anyhow!(
            "Failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn exec_in_session(session_id: &str, command: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(&["exec", session_id, "sh", "-c", command])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow!("{}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn stop_session(session_id: &str) -> Result<String> {
    Command::new("docker")
        .args(&["stop", session_id])
        .output()?;

    let mut sessions = DOCKER_SESSIONS.write().unwrap();
    if let Some(session) = sessions.get_mut(session_id) {
        session.status = SessionStatus::Stopped;
    }

    Ok(format!("Stopped: {}", session_id))
}

pub fn remove_session(session_id: &str) -> Result<String> {
    Command::new("docker")
        .args(&["rm", "-f", session_id])
        .output()?;

    let mut sessions = DOCKER_SESSIONS.write().unwrap();
    sessions.remove(session_id);

    Ok(format!("Removed: {}", session_id))
}

pub fn list_sessions() -> Vec<DockerSession> {
    let sessions = DOCKER_SESSIONS.read().unwrap();
    sessions.values().cloned().collect()
}

pub fn get_session(id: &str) -> Option<DockerSession> {
    let sessions = DOCKER_SESSIONS.read().unwrap();
    sessions.get(id).cloned()
}

pub fn docker_sessions_status() -> String {
    let sessions = DOCKER_SESSIONS.read().unwrap();
    format!("Active sessions: {}", sessions.len())
}

pub fn docker_commands() {
    println!("\n\x1b[36m🐳 Docker Sessions:\x1b[0m\n");
    println!("  \x1b[33mdevutils docker create <name>\x1b[0m    - Create isolated session");
    println!("  \x1b[33mdevutils docker exec <session> <cmd>\x1b[0m  - Run command in session");
    println!("  \x1b[33mdevutils docker stop <session>\x1b[0m   - Stop session");
    println!("  \x1b[33mdevutils docker rm <session>\x1b[0m     - Remove session");
    println!("  \x1b[33mdevutils docker list\x1b[0m             - List sessions");
    println!();
}
