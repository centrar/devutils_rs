//! Worktree Manager

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::process::Command;

static WORKTREES: Lazy<Mutex<HashMap<String, Worktree>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeStatus {
    Active,
    Busy,
    Completed,
    Failed,
    Locked,
    Removed,
}

#[derive(Debug, Clone)]
pub struct Worktree {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub status: WorktreeStatus,
}

pub fn list_worktrees() -> String {
    let worktrees = WORKTREES.lock().unwrap();
    if worktrees.is_empty() {
        return "[]".to_string();
    }
    let list: Vec<String> = worktrees
        .values()
        .map(|wt| format!("{{\"name\":\"{}\",\"path\":\"{}\",\"branch\":\"{}\",\"status\":\"{:?}\"}}", 
            wt.name, wt.path, wt.branch, wt.status))
        .collect();
    format!("[{}]", list.join(","))
}

pub fn create_worktree(name: &str, branch: &str) -> String {
    if name.is_empty() {
        return "Error: name required".to_string();
    }
    let branch_name = if branch.is_empty() {
        format!("dev/{}", name)
    } else {
        branch.to_string()
    };
    let worktree_path = format!(".worktrees/{}", name);
    
    if let Err(e) = fs::create_dir_all(&worktree_path) {
        return format!("Error creating directory: {}", e);
    }
    
    let result = Command::new("git")
        .args(&["worktree", "add", &worktree_path, &branch_name])
        .output();
    
    match result {
        Ok(output) => {
            if output.status.success() {
                WORKTREES.lock().unwrap().insert(
                    name.to_string(),
                    Worktree {
                        name: name.to_string(),
                        path: worktree_path.clone(),
                        branch: branch_name,
                        status: WorktreeStatus::Active,
                    },
                );
                format!("Created worktree: {} at {}", name, worktree_path)
            } else {
                let msg = String::from_utf8_lossy(&output.stderr).to_string();
                format!("Git error: {}", msg)
            }
        }
        Err(_) => {
            WORKTREES.lock().unwrap().insert(
                name.to_string(),
                Worktree {
                    name: name.to_string(),
                    path: worktree_path.clone(),
                    branch: branch_name,
                    status: WorktreeStatus::Active,
                },
            );
            format!("Created (local): {} at {}", name, worktree_path)
        }
    }
}

pub fn remove_worktree(name: &str) -> String {
    if name.is_empty() {
        return "Error: name required".to_string();
    }
    
    let worktree = {
        let mut worktrees = WORKTREES.lock().unwrap();
        worktrees.remove(name)
    };
    
    if let Some(wt) = worktree {
        let _ = Command::new("git")
            .args(&["worktree", "remove", &wt.path])
            .output();
        
        if let Err(e) = fs::remove_dir_all(&wt.path) {
            return format!("Removed from memory, but failed to remove directory: {}", e);
        }
        
        format!("Removed worktree: {}", name)
    } else {
        format!("Worktree not found: {}", name)
    }
}

pub fn run_in_worktree(name: &str, task: &str) -> String {
    if name.is_empty() {
        return "Error: name required".to_string();
    }
    if task.is_empty() {
        return "Error: task required".to_string();
    }
    
    let worktrees = WORKTREES.lock().unwrap();
    if let Some(wt) = worktrees.get(name) {
        let result = Command::new("sh")
            .args(&["-c", &format!("cd {} && {}", wt.path, task)])
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    format!("Running in {}: {}\n{}", wt.path, task, stdout)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    format!("Error in {}: {}", wt.path, stderr)
                }
            }
            Err(e) => format!("Failed to run: {}", e),
        }
    } else {
        format!("Worktree not found: {}", name)
    }
}

pub fn spawn_parallel_task(name: &str, task: &str) -> String {
    if name.is_empty() {
        return "Error: name required".to_string();
    }
    if task.is_empty() {
        return "Error: task required".to_string();
    }
    
    let worktrees = WORKTREES.lock().unwrap();
    if worktrees.get(name).is_some() {
        let task_owned = task.to_string();
        std::thread::spawn(move || {
            let _ = Command::new("sh")
                .args(&["-c", &task_owned])
                .output();
        });
        format!("Spawned parallel task in: {}", name)
    } else {
        format!("Worktree not found: {}", name)
    }
}

pub fn worktree_commands() -> String {
    "devutils worktree create/list/remove/run/spawn".to_string()
}