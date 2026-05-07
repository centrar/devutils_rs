//! Git Session Tracker - Track changes and session history

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static SESSIONS: Lazy<Mutex<Vec<GitSession>>> =
    Lazy::new(|| Mutex::new(load_sessions().unwrap_or_default()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSession {
    pub name: String,
    pub id: String,
    pub created_at: u64,
    pub description: String,
    pub changes: Vec<String>,
    pub command: String,
}

pub fn create_session(name: &str) -> String {
    let changes = match get_changed_files() {
        Ok(c) => c,
        Err(e) => return format!("Error getting changes: {}", e),
    };

    let session = GitSession {
        id: format!("sess_{}", now_ms()),
        name: name.to_string(),
        created_at: now_ms(),
        description: name.to_string(),
        changes: changes.clone(),
        command: String::new(),
    };

    if !changes.is_empty() {
        let _ = Command::new("git").args(["add", "-A"]).output();
    }

    match SESSIONS.lock() {
        Ok(mut sessions) => {
            sessions.push(session);
            if let Err(e) = save_sessions(&sessions) {
                return format!("Error saving sessions: {}", e);
            }
            format!("Created session '{}' with {} changes", name, changes.len())
        }
        Err(e) => format!("Error locking sessions: {}", e),
    }
}

pub fn list_sessions() -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if sessions.is_empty() {
                return "No sessions found".to_string();
            }
            let mut result = String::new();
            for s in sessions.iter() {
                result.push_str(&format!(
                    "{}: {} ({} changes)\n",
                    s.name, s.id, s.changes.len()
                ));
            }
            result
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn get_session(name: &str) -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if let Some(session) = sessions.iter().find(|s| s.name == name) {
                format!(
                    "{}: {} - {} files",
                    session.name, session.id, session.changes.len()
                )
            } else {
                format!("Session '{}' not found", name)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn view_session(name: &str) -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if let Some(session) = sessions.iter().find(|s| s.name == name) {
                let mut result = format!("Session: {}\n", session.name);
                result.push_str(&format!("ID: {}\n", session.id));
                result.push_str(&format!("Created: {}\n", session.created_at));
                result.push_str("Changes:\n");
                for change in &session.changes {
                    result.push_str(&format!("  {}\n", change));
                }
                result
            } else {
                format!("Session '{}' not found", name)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn switch_session(name: &str) -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if let Some(session) = sessions.iter().find(|s| s.name == name) {
                for path in &session.changes {
                    let _ = Command::new("git")
                        .args(["checkout", "--", path])
                        .output();
                }
                format!("Switched to session '{}'", name)
            } else {
                format!("Session '{}' not found", name)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn share_session(name: &str) -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if let Some(session) = sessions.iter().find(|s| s.name == name) {
                match serde_json::to_string(session) {
                    Ok(json) => json,
                    Err(e) => format!("Error serializing: {}", e),
                }
            } else {
                format!("Session '{}' not found", name)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn undo_session(name: &str) -> String {
    let mut sessions = match SESSIONS.lock() {
        Ok(s) => s,
        Err(e) => return format!("Error: {}", e),
    };

    if let Some(pos) = sessions.iter().position(|s| s.name == name) {
        let session = sessions.remove(pos);

        for path in &session.changes {
            let _ = Command::new("git")
                .args(["checkout", "--", path])
                .output();
        }

        if let Err(e) = save_sessions(&sessions) {
            return format!("Error saving: {}", e);
        }
        format!("Undone session '{}'", name)
    } else {
        format!("Session '{}' not found", name)
    }
}

pub fn export_session(name: &str, path: &str) -> String {
    match SESSIONS.lock() {
        Ok(sessions) => {
            if let Some(session) = sessions.iter().find(|s| s.name == name) {
                match serde_json::to_string_pretty(session) {
                    Ok(json) => {
                        match fs::write(path, &json) {
                            Ok(_) => format!("Exported to: {}", path),
                            Err(e) => format!("Error writing file: {}", e),
                        }
                    }
                    Err(e) => format!("Error serializing: {}", e),
                }
            } else {
                format!("Session '{}' not found", name)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn import_session(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<GitSession>(&content) {
            Ok(session) => match SESSIONS.lock() {
                Ok(mut sessions) => {
                    sessions.push(session);
                    "Session imported".to_string()
                }
                Err(e) => format!("Error: {}", e),
            },
            Err(e) => format!("Error parsing: {}", e),
        },
        Err(e) => format!("Error reading file: {}", e),
    }
}

fn get_changed_files() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| e.to_string())?;

    let status = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in status.lines() {
        if line.len() >= 3 {
            files.push(line[3..].trim().to_string());
        }
    }

    Ok(files)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn load_sessions() -> Result<Vec<GitSession>, String> {
    let path = PathBuf::from(".devutils_sessions.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let sessions: Vec<GitSession> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(sessions)
}

fn save_sessions(sessions: &[GitSession]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(sessions).map_err(|e| e.to_string())?;
    fs::write(".devutils_sessions.json", content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn git_session_commands() {
    println!("\n\x1b[36m📝 Git Sessions\x1b[0m");
    println!("\nUsage:");
    println!("  devutils session create <name>");
    println!("  devutils session list");
    println!("  devutils session get <name>");
    println!("  devutils session view <name>");
    println!("  devutils session switch <name>");
    println!("  devutils session share <name>");
    println!("  devutils session undo <name>");
}