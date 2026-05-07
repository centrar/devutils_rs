use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::ultimate_agent::{Step, AgentStatus};
use crate::ai::TokenUsage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCheckpoint {
    pub task: String,
    pub steps: Vec<Step>,
    pub status: AgentStatus,
    pub current_iteration: usize,
    pub total_usage: TokenUsage,
    pub workspace: PathBuf,
}

impl AgentCheckpoint {
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let checkpoint: Self = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(checkpoint)
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from(".devutils/agent_state.json")
    }
}
