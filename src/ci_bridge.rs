//! CI/CD Bridge - Self-healing pipeline integration
//! Automatically fixes CI failures by parsing logs and triggering the agent.

use crate::ultimate_agent::AutonomousAgent;
use std::path::PathBuf;

pub struct CiBridge {
    workspace: PathBuf,
}

impl CiBridge {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    /// Entry point for "devutils fix-ci"
    /// Parses a log file for errors and starts an autonomous fix session
    pub fn fix_ci_failure(&self, log_path: &str) -> Result<String, String> {
        let log_content = std::fs::read_to_string(log_path)
            .map_err(|e| format!("Failed to read CI log: {}", e))?;
        
        let task = format!(
            "Analyze and fix the following CI failure detected in the logs:\n\nLOGS:\n{}",
            log_content
        );

        let mut agent = AutonomousAgent::new(true)
            .map_err(|e| format!("Failed to initialize agent: {}", e))?;
        let state = agent.execute(&task).map_err(|e| e.to_string())?;
        
        if state.status == crate::ultimate_agent::AgentStatus::Completed {
            Ok("CI Failure successfully repaired and verified.".to_string())
        } else {
            Err("Failed to repair CI failure automatically.".to_string())
        }
    }
}
