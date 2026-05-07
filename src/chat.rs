//! AI Chat Module - Integrated Claude/GPT-powered chat in terminal

use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_HISTORY: usize = 100;
const CONTEXT_FILE: &str = ".devutils_context.json";

static CHAT_HISTORY: Lazy<Mutex<VecDeque<ChatMessage>>> =
    Lazy::new(|| Mutex::new(load_history().unwrap_or_default()));

static USER_PATTERNS: Lazy<Mutex<VecDeque<UserPattern>>> =
    Lazy::new(|| Mutex::new(load_patterns().unwrap_or_default()));

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
    pub context_files: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPattern {
    pub command: String,
    pub frequency: u32,
    pub last_used: u64,
    pub project: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskChain {
    pub steps: Vec<TaskStep>,
    pub current_step: usize,
    pub results: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskStep {
    pub description: String,
    pub command: String,
    pub status: StepStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct AIChat {
    pub client: crate::ai::AIClient,
    pub project_context: Vec<String>,
    pub current_task: Option<TaskChain>,
    pub translate_mode: bool,
}

impl AIChat {
    pub fn new() -> Self {
        let client = crate::ai::AIClient::new();

        let project_context = Self::detect_project_context();

        Self {
            client,
            project_context,
            current_task: None,
            translate_mode: false,
        }
    }

    fn detect_project_context() -> Vec<String> {
        let mut context = Vec::new();

        if let Ok(path) = std::env::current_dir() {
            for entry in fs::read_dir(&path).into_iter().flatten() {
                if let Ok(entry) = entry {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension() {
                            context.push(ext.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        context.sort();
        context.dedup();
        context
    }

    pub fn chat(&mut self, input: &str) -> Result<String> {
        let history = CHAT_HISTORY.lock().unwrap();

        let mut messages: Vec<ChatMessage> = history.iter().rev().take(10).cloned().collect();
        messages.reverse();

        let system_prompt = self.build_system_prompt();
        messages.insert(
            0,
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
                timestamp: 0,
                context_files: self.project_context.clone(),
            },
        );

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        });

        let response = self.client.chat_with_history(&messages)?;

        CHAT_HISTORY.lock().unwrap().push_back(ChatMessage {
            role: "user".to_string(),
            content: input.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        });

        CHAT_HISTORY.lock().unwrap().push_back(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        });

        save_history(&CHAT_HISTORY.lock().unwrap())?;

        Ok(response)
    }

    fn build_system_prompt(&self) -> String {
        let patterns = USER_PATTERNS.lock().unwrap();
        let top_patterns: Vec<_> = patterns.iter().take(5).collect();

        let mut prompt = String::from(
            r#"You are DevUtils AI - an advanced AI coding assistant integrated into a developer CLI tool. 

Capabilities:
- Write, explain, debug, and refactor code
- Execute shell commands safely
- Search and analyze codebases
- Run tests and builds
- Work with Git

Be concise, helpful, and think step-by-step for complex tasks.
"#,
        );

        if !top_patterns.is_empty() {
            prompt.push_str("\n\nUser patterns I'm learning:\n");
            for p in top_patterns {
                prompt.push_str(&format!("- {} (used {} times)\n", p.command, p.frequency));
            }
        }

        if !self.project_context.is_empty() {
            prompt.push_str(&format!(
                "\nProject file types: {:?}\n",
                self.project_context
            ));
        }

        prompt
    }

    pub fn translate(&mut self, natural_language: &str) -> Result<String> {
        let prompt = format!(
            r#"Translate this natural language request into a shell command or series of shell commands.
Return ONLY the command(s) to execute, no explanation. For multiple commands, use && to chain.
If the request is ambiguous or impossible, return exactly: "IMPOSSIBLE: <reason>"

Request: {}"#,
            natural_language
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        }];

        let response = self.client.chat_with_history(&messages)?;

        Ok(response.trim().to_string())
    }

    pub fn execute_task_chain(&mut self, task_description: &str) -> Result<String> {
        let prompt = format!(
            r#"Break down this task into executable steps. Return a JSON array of steps.
Each step should have: description (what to do) and command (shell command to run).

Task: {}

Respond with JSON like: [{{"description": "step 1", "command": "echo test"}}, ...]"#,
            task_description
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        }];

        let response = self.client.chat_with_history(&messages)?;

        let steps: Vec<TaskStep> = serde_json::from_str(&response).unwrap_or_else(|_| {
            vec![TaskStep {
                description: task_description.to_string(),
                command: format!("echo {}", task_description),
                status: StepStatus::Pending,
                output: None,
            }]
        });

        self.current_task = Some(TaskChain {
            current_step: 0,
            steps,
            results: vec![],
        });

        self.run_next_step()
    }

    pub fn run_next_step(&mut self) -> Result<String> {
        let task = match &mut self.current_task {
            Some(t) => t,
            None => return Ok("No active task chain".to_string()),
        };

        while task.current_step < task.steps.len() {
            let step = &mut task.steps[task.current_step];

            step.status = StepStatus::Running;

            let output = std::process::Command::new("powershell")
                .args(["-Command", &step.command])
                .output()?;

            step.output = Some(String::from_utf8_lossy(&output.stdout).to_string());
            step.status = if output.status.success() {
                StepStatus::Completed
            } else {
                StepStatus::Failed
            };

            task.results.push(step.output.clone().unwrap_or_default());
            task.current_step += 1;

            if matches!(step.status, StepStatus::Failed) {
                return Ok(format!(
                    "Step {} failed: {}\nStopping task chain.",
                    task.current_step, step.description
                ));
            }
        }

        let summary = task.results.join("\n---\n");
        Ok(format!("Task chain completed!\n\n{}", summary))
    }

    pub fn search_codebase(&mut self, question: &str) -> Result<String> {
        let context = self.gather_codebase_context()?;

        let prompt = format!(
            "Based on this codebase context:\n\n{}\n\nAnswer: {}",
            context, question
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context_files: vec![],
        }];

        self.client.chat_with_history(&messages)
    }

    fn gather_codebase_context(&self) -> Result<String> {
        let mut context = String::new();

        if let Ok(path) = std::env::current_dir() {
            let files: Vec<_> = fs::read_dir(&path)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| {
                            matches!(
                                ext.to_str(),
                                Some("rs") | Some("py") | Some("js") | Some("ts") | Some("go")
                            )
                        })
                        .unwrap_or(false)
                })
                .take(20)
                .collect();

            for entry in files {
                let content = fs::read_to_string(entry.path())
                    .map(|c| c.lines().take(50).collect::<Vec<_>>().join("\n"))
                    .unwrap_or_default();
                context.push_str(&format!("\n\n// {} ---\n", entry.path().display()));
                context.push_str(&content);
            }
        }

        Ok(context)
    }

    pub fn record_pattern(&self, command: &str) {
        let mut patterns = USER_PATTERNS.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(p) = patterns.iter_mut().find(|p| p.command == command) {
            p.frequency += 1;
            p.last_used = now;
        } else {
            patterns.push_back(UserPattern {
                command: command.to_string(),
                frequency: 1,
                last_used: now,
                project: std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())),
            });
        }

        if patterns.len() > MAX_HISTORY {
            patterns.pop_front();
        }

        let _ = save_patterns(&patterns);
    }

    pub fn get_suggestions(&self) -> Vec<String> {
        let patterns = USER_PATTERNS.lock().unwrap();

        patterns
            .iter()
            .filter(|p| p.frequency > 2)
            .take(5)
            .map(|p| p.command.clone())
            .collect()
    }
}

fn load_history() -> Result<VecDeque<ChatMessage>> {
    let path = PathBuf::from(CONTEXT_FILE);
    if !path.exists() {
        return Ok(VecDeque::new());
    }

    let content = fs::read_to_string(&path)?;
    let messages: Vec<ChatMessage> = serde_json::from_str(&content)?;
    Ok(VecDeque::from(messages))
}

fn save_history(history: &VecDeque<ChatMessage>) -> Result<()> {
    let history: Vec<_> = history.iter().take(MAX_HISTORY).cloned().collect();
    let content = serde_json::to_string_pretty(&history)?;
    fs::write(CONTEXT_FILE, content)?;
    Ok(())
}

fn load_patterns() -> Result<VecDeque<UserPattern>> {
    let path = PathBuf::from(".devutils_patterns.json");
    if !path.exists() {
        return Ok(VecDeque::new());
    }

    let content = fs::read_to_string(&path)?;
    let patterns: Vec<UserPattern> = serde_json::from_str(&content)?;
    Ok(VecDeque::from(patterns))
}

fn save_patterns(patterns: &VecDeque<UserPattern>) -> Result<()> {
    let patterns: Vec<_> = patterns.iter().take(MAX_HISTORY).cloned().collect();
    let content = serde_json::to_string_pretty(&patterns)?;
    fs::write(".devutils_patterns.json", content)?;
    Ok(())
}

pub fn enter_chat_mode() -> Result<()> {
    use std::io::{self, Write};

    println!("\n\x1b[36m💬 DevUtils AI Chat Mode\x1b[0m");
    println!("Type 'quit' to exit, 'help' for commands, 'translate' for natural language mode\n");

    let mut chat = AIChat::new();
    let mut input = String::new();

    loop {
        print!("\x1b[32m>\x1b[0m ");
        io::stdout().flush()?;

        input.clear();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "quit" | "exit" | "q" => {
                println!("\x1b[33m👋 Chat session saved\x1b[0m");
                break;
            }
            "help" | "?" => {
                println!("Commands:");
                println!("  translate <nl> - Convert natural language to shell command");
                println!("  task <desc>  - Execute multi-step task");
                println!("  search <q>  - Ask about codebase");
                println!("  suggest    - Show learned suggestions");
                println!("  clear      - Clear chat history");
                continue;
            }
            "translate" => {
                chat.translate_mode = true;
                println!("\x1b[33m🌍 Translate mode - enter natural language\x1b[0m");
                continue;
            }
            "suggest" => {
                let suggestions = chat.get_suggestions();
                if suggestions.is_empty() {
                    println!("No suggestions yet - use commands to build patterns");
                } else {
                    println!("\x1b[33m📝 Suggestions:\x1b[0m");
                    for s in suggestions {
                        println!("  - {}", s);
                    }
                }
                continue;
            }
            "clear" => {
                CHAT_HISTORY.lock().unwrap().clear();
                println!("\x1b[32m✅ History cleared\x1b[0m");
                continue;
            }
            _ => {}
        }

        if chat.translate_mode {
            chat.translate_mode = false;
            match chat.translate(input) {
                Ok(cmd) => {
                    println!("\x1b[33m→ \x1b[0m{}", cmd);
                    print!("\x1b[36mExecute? [y/N]\x1b[0m ");
                    io::stdout().flush()?;

                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm)?;

                    if confirm.trim().to_lowercase() == "y" {
                        let output = std::process::Command::new("powershell")
                            .args(["-Command", &cmd])
                            .output()?;
                        println!("\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                Err(e) => println!("\x1b[31mError: {}\x1b[0m", e),
            }
        } else if input.starts_with("task ") {
            let task = &input[5..];
            match chat.execute_task_chain(task) {
                Ok(result) => println!("\x1b[32m{}\x1b[0m", result),
                Err(e) => println!("\x1b[31mError: {}\x1b[0m", e),
            }
        } else if input.starts_with("search ") {
            let query = &input[7..];
            match chat.search_codebase(query) {
                Ok(result) => println!("\x1b[36m{}\x1b[0m", result),
                Err(e) => println!("\x1b[31mError: {}\x1b[0m", e),
            }
        } else {
            match chat.chat(input) {
                Ok(response) => {
                    println!("\x1b[36m{}\x1b[0m", response);
                }
                Err(e) => {
                    println!("\x1b[31mError: {}\x1b[0m", e);
                }
            }
        }
    }

    Ok(())
}

pub fn enter_translate_mode(nl: &str) -> Result<String> {
    let mut chat = AIChat::new();
    chat.translate(nl)
}

pub fn execute_task(task: &str) -> Result<String> {
    let mut chat = AIChat::new();
    chat.execute_task_chain(task)
}

pub fn search_codebase(question: &str) -> Result<String> {
    let mut chat = AIChat::new();
    chat.search_codebase(question)
}
