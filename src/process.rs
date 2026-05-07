//! Process Manager - background job execution

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Mutex;

static PROCESSES: Lazy<Mutex<HashMap<String, ProcessInfo>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub command: String,
    pub working_dir: String,
    pub started_at: u64,
    pub status: ProcessStatus,
    pub pid: u32,
    pub output_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Running,
    Completed,
    Failed,
}

pub fn spawn_background(command: &str) -> Result<String, String> {
    let id = format!(
        "proc_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let mut cmd = Command::new("powershell");
    cmd.args(["-Command", command]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| e.to_string())?;
    let pid = child.id();

    let info = ProcessInfo {
        id: id.clone(),
        command: command.to_string(),
        working_dir: cwd,
        started_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        status: ProcessStatus::Running,
        pid,
        output_lines: vec![],
    };

    PROCESSES.lock().unwrap().insert(id.clone(), info);

    Ok(format!("Started process {} (PID: {})", id, pid))
}

pub fn list_processes() -> Vec<ProcessInfo> {
    PROCESSES.lock().unwrap().values().cloned().collect()
}

pub fn kill_process(id: &str) -> Result<String, String> {
    let mut procs = PROCESSES.lock().unwrap();

    if let Some(info) = procs.get_mut(id) {
        info.status = ProcessStatus::Failed;

        #[cfg(target_os = "windows")]
        let _ = Command::new("taskkill")
            .args(["/PID", &info.pid.to_string(), "/F"])
            .output();
        #[cfg(not(target_os = "windows"))]
        let _ = Command::new("kill")
            .arg("-9")
            .arg(&info.pid.to_string())
            .output();

        procs.remove(id);
        return Ok(format!("Killed process: {}", id));
    }

    Err(format!("Process '{}' not found", id))
}

pub fn cleanup_finished() -> usize {
    let mut procs = PROCESSES.lock().unwrap();
    let initial = procs.len();
    procs.retain(|_, p| p.status == ProcessStatus::Running);
    initial - procs.len()
}
