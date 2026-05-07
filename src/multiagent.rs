//! Multiagent Orchestration System - Real Parallel Execution

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use crate::error::{DevUtilsError, Result};
use crate::ai::AIClient;

static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("devutils");
    let _ = fs::create_dir_all(&dir);
    dir
});

static AGENTS_FILE: Lazy<PathBuf> = Lazy::new(|| DATA_DIR.join("agents.json"));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStore {
    pub agents: HashMap<String, AgentInfo>,
    pub counter: u64,
}

impl AgentStore {
    fn new() -> Self {
        Self {
            agents: HashMap::new(),
            counter: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub task: String,
    pub state: String,
    pub created_at: u64,
    pub tools_used: usize,
}

fn load_store() -> AgentStore {
    if AGENTS_FILE.exists() {
        if let Ok(content) = fs::read_to_string(&*AGENTS_FILE) {
            if let Ok(store) = serde_json::from_str(&content) {
                return store;
            }
        }
    }
    AgentStore::default()
}

fn save_store(store: &AgentStore) {
    if let Ok(content) = serde_json::to_string_pretty(store) {
        let _ = fs::write(&*AGENTS_FILE, content);
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn spawn_multiagent(name: &str, task: String) -> String {
    let mut store = load_store();
    store.counter += 1;
    let id = format!("{}_{}", name, store.counter);

    let info = AgentInfo {
        id: id.clone(),
        name: name.to_string(),
        task: task.clone(),
        state: "Running".to_string(),
        created_at: now_millis(),
        tools_used: 0,
    };

    store.agents.insert(id.clone(), info);
    save_store(&store);
    format!("Spawned agent {} with task: {}", id, task)
}

/// Execute a list of subtasks in parallel using Tokio
pub fn execute_parallel_tasks(tasks: Vec<String>) -> Result<Vec<String>> {
    let rt = Runtime::new().map_err(|e| DevUtilsError::IoError(e))?;
    
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for (i, task) in tasks.into_iter().enumerate() {
            handles.push(tokio::spawn(async move {
                let client = AIClient::new();
                let prompt = format!("Execute subtask {}:\n{}", i+1, task);
                match client.generate(&prompt) {
                    Ok((res, _)) => res,
                    Err(e) => format!("Subtask {} failed: {}", i+1, e)
                }
            }));
        }
        
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(res) => results.push(res),
                Err(e) => results.push(format!("Task execution panicked: {}", e))
            }
        }
        
        Ok(results)
    })
}

pub fn list_multiagents() -> String {
    let store = load_store();
    if store.agents.is_empty() {
        return "No active agents".to_string();
    }

    let mut output = String::from("Active Agents:\n");
    for (id, info) in &store.agents {
        output.push_str(&format!(
            "  {} - {} (state: {})\n",
            id, info.name, info.state
        ));
    }
    output
}

pub fn get_multiagent_stats(id: &str) -> String {
    let store = load_store();
    if let Some(info) = store.agents.get(id) {
        format!(
            "Agent: {}\n  Name: {}\n  Task: {}\n  State: {}\n  Tools Used: {}\n  Created: {}",
            info.id, info.name, info.task, info.state, info.tools_used, info.created_at
        )
    } else {
        format!("Agent not found: {}", id)
    }
}

pub fn multiagent_commands() {
    println!("\nMultiagent Commands:");
    println!("  spawn <name> <task>  - Spawn new agent");
    println!("  list                - List all agents");
    println!("  stats <id>         - Get agent stats");
}
