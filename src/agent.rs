//! Agent module - AI agent management (stubs for backward compatibility)

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub task: String,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
}

pub fn list_agents() -> Vec<Agent> {
    Vec::new()
}

pub fn create_agent(_name: &str, _task: &str) -> Result<String, String> {
    Ok("agent-id-1".to_string())
}

pub fn kill_agent(_id: &str) -> Result<String, String> {
    Ok("Agent killed".to_string())
}

pub fn spawn_parallel_agents(_tasks: Vec<String>) -> Vec<String> {
    Vec::new()
}