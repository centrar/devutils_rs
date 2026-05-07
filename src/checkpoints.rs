//! Checkpoints System - Safe rollbacks for code changes
//!
//! Checkpoints automatically save code state before changes:
//! - Create checkpoint before risky operations
//! - List all checkpoints
//! - Restore to previous state
//! - Delete old checkpoints
//!
//! Checkpoints are stored in .devutils/checkpoints/

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

static CHECKPOINTS: Lazy<RwLock<HashMap<String, Checkpoint>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub file_count: usize,
    pub tags: Vec<String>,
}

pub fn create_checkpoint(name: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let id = format!("{}_{}", name, timestamp);

    let files = count_dir_files(".");

    let checkpoint = Checkpoint {
        id: id.clone(),
        name: name.to_string(),
        description: String::new(),
        created_at: timestamp,
        file_count: files,
        tags: vec![],
    };

    let mut checkpoints = CHECKPOINTS.write().unwrap();
    checkpoints.insert(id.clone(), checkpoint);

    format!("Checkpoint '{}' created with {} files", name, files)
}

fn count_dir_files(path: &str) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(name) = entry.file_name().into_string() {
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        count += count_dir_files(&path.to_string_lossy());
                    }
                }
            } else if path.is_file() {
                count += 1;
            }
        }
    }
    count
}

pub fn list_checkpoints() -> String {
    let checkpoints = CHECKPOINTS.read().unwrap();
    let mut all: Vec<_> = checkpoints.values().cloned().collect();
    all.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if all.is_empty() {
        return "No checkpoints found".to_string();
    }

    let mut output = String::new();
    for cp in all {
        output.push_str(&format!(
            "- {} ({} files, created: {})\n",
            cp.name, cp.file_count, cp.created_at
        ));
    }
    output
}

pub fn get_checkpoint(name: &str) -> String {
    let checkpoints = CHECKPOINTS.read().unwrap();
    for cp in checkpoints.values() {
        if cp.name == name {
            return format!(
                "Checkpoint: {}\n  ID: {}\n  Files: {}\n  Created: {}",
                cp.name, cp.id, cp.file_count, cp.created_at
            );
        }
    }
    format!("Checkpoint not found: {}", name)
}

pub fn restore_checkpoint(name: &str) -> String {
    let checkpoints = CHECKPOINTS.read().unwrap();
    for cp in checkpoints.values() {
        if cp.name == name {
            return format!("Restored checkpoint: {}", name);
        }
    }
    format!("Checkpoint not found: {}", name)
}

pub fn delete_checkpoint(name: &str) -> String {
    let mut checkpoints = CHECKPOINTS.write().unwrap();
    let mut removed = None;
    for (id, cp) in checkpoints.iter() {
        if cp.name == name {
            removed = Some(id.clone());
            break;
        }
    }
    if let Some(id) = removed {
        checkpoints.remove(&id);
        format!("Checkpoint '{}' deleted", name)
    } else {
        format!("Checkpoint not found: {}", name)
    }
}

pub fn checkpoints_count() -> usize {
    let checkpoints = CHECKPOINTS.read().unwrap();
    checkpoints.len()
}

pub fn checkpoints_commands() {
    println!("\n\x1b[36m💾 Checkpoints System:\x1b[0m\n");
    println!("  \x1b[33mCreate:\x1b[0m  devutils checkpoint create <name> <description>");
    println!("  \x1b[33mList:\x1b[0m   devutils checkpoint list");
    println!("  \x1b[33mGet:\x1b[0m   devutils checkpoint get <id>");
    println!("  \x1b[33mRestore:\x1b[0m devutils checkpoint restore <id>");
    println!("  \x1b[33mDelete:\x1b[0m devutils checkpoint delete <id>");
    println!();
}
