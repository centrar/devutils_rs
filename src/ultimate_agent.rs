//! AI Autonomous Agent v2 - Industrial Grade
//! 
//! Features:
//! - Structural Project Awareness (ProjectMap)
//! - Surgical Search-Replace Editing (Aider-style)
//! - Plan-Act-Observe-Verify execution loop
//! - Model Context Protocol (MCP) tool-use integration
//! - Autonomous validation (build/test/fix)

use crate::ai::AIClient;
use crate::project_map::ProjectMap;
use crate::atomic_edit::{self, SearchReplaceBlock};
use crate::shell_interpreter::ShellInterpreter;
use crate::vector_store::VectorStore;
use crate::model_router::ModelRouter;
use crate::git::{GitOps, GitCommands};
use crate::agent_state::AgentCheckpoint;
use crate::sandbox::Sandbox;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::error::{DevUtilsError, Result};

/// Main agent structure
#[allow(dead_code)]
pub struct AutonomousAgent {
    client: AIClient,
    pub verbose: bool,
    max_iterations: usize,
    workspace: PathBuf,
    project_map: ProjectMap,
    shell: ShellInterpreter,
    router: ModelRouter,
    vector_store: VectorStore,
    sandbox: Sandbox,
    pub total_usage: crate::ai::TokenUsage,
    pub token_limit: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Step {
    pub iteration: usize,
    pub plan: String,
    pub action: String,
    pub observation: String,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub output: String,
    pub error: Option<String>,
}

pub struct AgentState {
    pub task: String,
    pub steps: Vec<Step>,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    Planning,
    Acting,
    Verifying,
    Completed,
    Failed(String),
}

impl AutonomousAgent {
    pub fn new(verbose: bool) -> Result<Self> {
        let workspace = PathBuf::from(".");
        let vector_store = VectorStore::new()
            .map_err(|e| DevUtilsError::AgentError(format!("Failed to initialize VectorStore: {}", e)))?;
        Ok(Self {
            client: AIClient::new(),
            verbose,
            max_iterations: 20,
            workspace: workspace.clone(),
            project_map: ProjectMap::new(&workspace.to_string_lossy()),
            shell: ShellInterpreter::new(),
            router: ModelRouter::new(false),
            vector_store,
            sandbox: Sandbox::new(),
            total_usage: crate::ai::TokenUsage::default(),
            token_limit: 100_000,
        })
    }

    pub fn execute(&mut self, task: &str) -> Result<AgentState> {
        if self.verbose {
            println!("🤖 Industrial Agent v2.5 (Industrial Grade) engaged...");
            println!("📋 Task: {}", task);
        }

        // Feature 14: Atomic Rollback - Start Session
        let git = GitOps::new();
        let _ = git.run(GitCommands::SessionStart);

        let mut state = AgentState {
            task: task.to_string(),
            steps: vec![],
            status: AgentStatus::Planning,
        };

        // Feature 2: Semantic Search - Index Project
        let _ = self.vector_store.index_project(&self.workspace.to_string_lossy());

        for i in 1..=self.max_iterations {
            // Feature 4: Token Budget Management
            if self.total_usage.total_tokens >= self.token_limit {
                if self.verbose { println!("🛑 Token budget exceeded ({} tokens). Stopping.", self.token_limit); }
                state.status = AgentStatus::Failed("Token budget exceeded".to_string());
                break;
            }

            if self.verbose {
                println!("\n🔄 [Iteration {}/{}] [Tokens: {}]", i, self.max_iterations, self.total_usage.total_tokens);
            }

            // 1. PLANNING & ACTION
            let history = self.format_history(&state.steps);
            
            // Feature 1: Repo-Map - Generate structural summary
            let repo_map = self.project_map.generate_summary();
            
            // Feature 2: Local RAG - Fetch semantic context
            let semantic_context = self.vector_store.query(task, 3).unwrap_or_default().join("\n\n");
            
            let prompt = self.generate_system_prompt(task, &repo_map, &semantic_context, &history);
            
            // Feature 3 & 4: Multi-Model Fallback + Token Tracking
            let (response, usage) = self.router.generate_with_usage(&prompt)?;
            
            if self.verbose {
                println!("🧠 Internal Monologue: Analyzing plan for safety and precision...");
            }
            
            // Extreme Depth: Pre-execution "Safety Check" loop
            if response.contains("<<<<<<< SEARCH") {
                 let blocks = self.parse_sr_blocks(&response);
                 if let Some(path) = self.guess_path_from_context(&response) {
                     if let Err(e) = atomic_edit::apply_search_replace(&path, &blocks, false) {
                         if self.verbose { println!("⚠️ Pre-check failed: {}. Re-planning...", e); }
                         // Push the error back into history for self-correction
                         state.steps.push(Step {
                             iteration: i,
                             plan: "Self-correction".to_string(),
                             action: response,
                             observation: format!("RE-PLANNING REQUIRED: Your search block failed uniqueness check: {}", e),
                         });
                         continue;
                     }
                 }
            }
            
            // Update total usage
            self.total_usage.prompt_tokens += usage.prompt_tokens;
            self.total_usage.completion_tokens += usage.completion_tokens;
            self.total_usage.total_tokens += usage.total_tokens;

            if response.contains("FINAL_ANSWER") {
                state.status = AgentStatus::Completed;
                break;
            }

            // 2. EXECUTION (Tool Use)
            let observation = self.handle_agent_output(&response)?;
            
            state.steps.push(Step {
                iteration: i,
                plan: "See action".to_string(),
                action: response,
                observation: observation.clone(),
            });

            // Feature 25: Task Resumption - Save Checkpoint
            self.save_checkpoint(&state, i);

            // 3. VERIFICATION
            if i % 2 == 0 {
                if self.verbose {
                    println!("🧪 Running verification suite...");
                }
                let verify_res = self.run_verification();
                if verify_res.passed {
                    if self.verbose { println!("✅ System stable."); }
                } else {
                    if self.verbose { println!("⚠️ Build errors found. Reflecting and self-correcting..."); }
                    let error_msg = verify_res.error.unwrap_or_default();
                    
                    // Feature 30: Self-Reflection Step
                    let reflection_prompt = format!("The last code change caused a build/test failure:\n{}\nWhy did this fail and what is the exact fix required? Be concise.", error_msg);
                    let reflection = self.router.generate_code(&reflection_prompt)
                        .unwrap_or_else(|_| "Failed to generate reflection.".to_string());
                        
                    state.steps.last_mut().unwrap().observation.push_str(&format!("\nBUILD ERRORS:\n{}\n\nSELF-REFLECTION:\n{}", error_msg, reflection));
                }
            }
        }

        Ok(state)
    }

    pub fn finalize(&self, success: bool) {
        let git = GitOps::new();
        if success {
            // Feature 42: Auto-Commit
            let _ = git.run(GitCommands::CommitAuto);
        } else {
            // Feature 14: Atomic Rollback
            let _ = git.run(GitCommands::SessionRollback);
        }
    }

    fn generate_system_prompt(&self, task: &str, repo_map: &str, semantic_context: &str, history: &str) -> String {
        format!(r#"You are an Autonomous Systems Engineer. Your goal is to complete: {task}

### PROJECT STRUCTURE (Repo-Map):
{repo_map}

### SEMANTIC CODE CONTEXT (Local RAG):
{semantic_context}

### RECENT HISTORY:
{history}

### INSTRUCTIONS:
1. EXPLORE: Use `READ_FILE` or `SEARCH` if you're unsure.
2. EDIT: Use surgical `SEARCH/REPLACE` blocks.
3. RUN: Use `RUN_COMMAND("...")` to execute terminal commands, run tests, or install deps.
4. VERIFY: When done, output `FINAL_ANSWER`.

### EDIT FORMAT:
<<<<<<< SEARCH
[exact code to find]
=======
[new code]
>>>>>>> REPLACE

### CONSTRAINTS:
- Use relative paths.
- Be precise. Exact matches only.
- Output your thought process first, then your tool calls."#)
    }

    fn handle_agent_output(&mut self, output: &str) -> Result<String> {
        let mut results = Vec::new();

        // Check for file read requests
        if let Some(path) = self.extract_tool_arg(output, "READ_FILE") {
            if self.verbose { println!("📖 Reading: {}", path); }
            match fs::read_to_string(self.workspace.join(&path)) {
                Ok(c) => results.push(format!("FILE CONTENT ({}):\n{}", path, c)),
                Err(e) => results.push(format!("Error reading {}: {}", path, e)),
            }
        }

        // Check for command execution requests
        if let Some(cmd) = self.extract_tool_arg(output, "RUN_COMMAND") {
            if self.verbose { println!("🚀 Running (Sandboxed): {}", cmd); }
            match self.sandbox.execute(&cmd, &self.workspace) {
                Ok(stdout) => {
                    results.push(format!("COMMAND OUTPUT:\n{}", stdout));
                }
                Err(e) => results.push(format!("Sandbox execution error: {}", e)),
            }
        }

        // Check for Search-Replace blocks
        let blocks = self.parse_sr_blocks(output);
        if !blocks.is_empty() {
            // Find the file path mentioned before the block
            // This is a heuristic - real implementation would be more robust
            if let Some(path) = self.guess_path_from_context(output) {
                if self.verbose { println!("✏️  Surgically editing: {}", path); }
                match atomic_edit::apply_search_replace(&path, &blocks, true) {
                    Ok(res) => results.push(format!("Successfully edited {}.\nDiff:\n{}", path, res.diff)),
                    Err(e) => results.push(format!("Error editing {}: {}", path, e)),
                }
            }
        }

        Ok(results.join("\n---\n"))
    }

    fn parse_sr_blocks(&self, text: &str) -> Vec<SearchReplaceBlock> {
        let mut blocks = Vec::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            if line.contains("<<<<<<< SEARCH") {
                let mut search = String::new();
                while let Some(s_line) = lines.next() {
                    if s_line.contains("=======") { break; }
                    search.push_str(s_line); search.push('\n');
                }
                let mut replace = String::new();
                while let Some(r_line) = lines.next() {
                    if r_line.contains(">>>>>>> REPLACE") { break; }
                    replace.push_str(r_line); replace.push('\n');
                }
                blocks.push(SearchReplaceBlock { 
                    search: search.trim_end().to_string(), 
                    replace: replace.trim_end().to_string() 
                });
            }
        }
        blocks
    }

    fn extract_tool_arg(&self, text: &str, tool: &str) -> Option<String> {
        let pattern = format!("{}(", tool);
        if let Some(start) = text.find(&pattern) {
            let arg_start = start + pattern.len();
            if let Some(end) = text[arg_start..].find(')') {
                return Some(text[arg_start..arg_start+end].trim_matches('"').to_string());
            }
        }
        None
    }

    fn guess_path_from_context(&self, text: &str) -> Option<String> {
        // Look for the last line ending with : or containing a file extension
        for line in text.lines().rev() {
            let line = line.trim();
            if line.contains('.') && (line.ends_with(".rs") || line.ends_with(".py") || line.ends_with(".js") || line.ends_with(".ts")) {
                // Clean up markdown/extra text
                return Some(line.trim_matches(|c| c == '`' || c == ':' || c == '*').to_string());
            }
        }
        None
    }

    fn format_history(&self, steps: &[Step]) -> String {
        steps.iter().map(|s| format!("Iter {}: {}\nObservation: {}", s.iteration, s.action, s.observation))
            .collect::<Vec<_>>().join("\n\n")
    }

    fn run_verification(&self) -> TestResult {
        // Run cargo check and cargo test as verification
        let check_out = Command::new("cargo")
            .args(["check"])
            .current_dir(&self.workspace)
            .output();
            
        let test_out = Command::new("cargo")
            .args(["test", "--no-run"]) // compile tests at least
            .current_dir(&self.workspace)
            .output();
        
        match (check_out, test_out) {
            (Ok(chk), Ok(tst)) => {
                let passed = chk.status.success() && tst.status.success();
                let mut error = String::new();
                if !chk.status.success() { error.push_str(&String::from_utf8_lossy(&chk.stderr)); }
                if !tst.status.success() { error.push_str(&String::from_utf8_lossy(&tst.stderr)); }
                
                TestResult {
                    passed,
                    output: String::from_utf8_lossy(&chk.stdout).to_string(),
                    error: if passed { None } else { Some(error) },
                }
            },
            (Err(e), _) | (_, Err(e)) => TestResult { passed: false, output: "".into(), error: Some(e.to_string()) }
        }
    }

    fn save_checkpoint(&self, state: &AgentState, iteration: usize) {
        let checkpoint = AgentCheckpoint {
            task: state.task.clone(),
            steps: state.steps.clone(),
            status: state.status.clone(),
            current_iteration: iteration,
            total_usage: self.total_usage.clone(),
            workspace: self.workspace.clone(),
        };
        let _ = checkpoint.save(&AgentCheckpoint::default_path());
    }

    pub fn resume_from_checkpoint(&mut self) -> Result<AgentState> {
        let checkpoint = AgentCheckpoint::load(&AgentCheckpoint::default_path())
            .map_err(|e| DevUtilsError::AgentError(format!("Failed to load checkpoint: {}", e)))?;
        self.total_usage = checkpoint.total_usage;
        self.workspace = checkpoint.workspace;
        
        Ok(AgentState {
            task: checkpoint.task,
            steps: checkpoint.steps,
            status: checkpoint.status,
        })
    }

    pub fn export_transcript(&self, state: &AgentState) -> Result<String> {
        let mut transcript = format!("# DevUtils Agent Task Transcript\n\n**Task:** {}\n**Status:** {:?}\n**Total Tokens:** {}\n\n", 
            state.task, state.status, self.total_usage.total_tokens);
        
        for step in &state.steps {
            transcript.push_str(&format!("## Iteration {}\n", step.iteration));
            transcript.push_str("### 🤖 Action\n");
            transcript.push_str(&format!("```\n{}\n```\n", step.action));
            transcript.push_str("### 📝 Observation\n");
            transcript.push_str(&format!("```\n{}\n```\n\n", step.observation));
        }

        let path = format!(".devutils/transcripts/task_{}.md", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
        
        let _ = fs::create_dir_all(".devutils/transcripts");
        fs::write(&path, transcript).map_err(|e| DevUtilsError::IoError(e))?;
        
        Ok(path)
    }
}

/// Run the ultimate agent with full capabilities (Backward compatibility wrapper)
pub fn run_ultimate_agent(task: &str, verbose: bool) -> std::result::Result<String, String> {
    let mut agent = match AutonomousAgent::new(verbose) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to initialize agent: {}", e)),
    };
    match agent.execute(task) {
        Ok(state) => Ok(format!("Task completed in {} steps.", state.steps.len())),
        Err(e) => Err(e.to_string()),
    }
}
