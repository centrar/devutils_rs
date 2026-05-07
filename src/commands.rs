//! Custom Commands - Define your own /commands like OpenCode

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const COMMANDS_DIR: &str = ".devutils/commands";
const COMMANDS_DB: &str = ".devutils/commands.json";

static CUSTOM_COMMANDS: Lazy<Mutex<HashMap<String, CustomCommand>>> =
    Lazy::new(|| Mutex::new(load_commands().unwrap_or_default()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub agent: Option<String>,
    pub model: Option<String>,
}

impl CustomCommand {
    pub fn new(name: &str, description: &str, prompt: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            prompt: prompt.to_string(),
            agent: None,
            model: None,
        }
    }
}

pub fn add_command(name: &str, description: &str, prompt: &str) -> Result<String, String> {
    let cmd = CustomCommand::new(name, description, prompt);
    let cmd_name = cmd.name.clone();

    CUSTOM_COMMANDS
        .lock()
        .unwrap()
        .insert(cmd_name.clone(), cmd);
    save_commands(&CUSTOM_COMMANDS.lock().unwrap())?;

    Ok(format!("Added command: /{}", cmd_name))
}

pub fn list_commands() -> Vec<CustomCommand> {
    let commands = CUSTOM_COMMANDS.lock().unwrap();
    let mut list: Vec<_> = commands.values().cloned().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

pub fn get_command(name: &str) -> Option<CustomCommand> {
    CUSTOM_COMMANDS.lock().unwrap().get(name).cloned()
}

pub fn remove_command(name: &str) -> Result<String, String> {
    let mut commands = CUSTOM_COMMANDS.lock().unwrap();

    if commands.remove(name).is_some() {
        drop(commands);
        save_commands(&CUSTOM_COMMANDS.lock().unwrap())?;
        return Ok(format!("Removed command: /{}", name));
    }

    Err(format!("Command '/{}' not found", name))
}

pub fn run_command(name: &str, args: &str) -> Result<String, String> {
    let cmd = get_command(name).ok_or_else(|| format!("Command '/{}' not found", name))?;

    let prompt = if args.is_empty() {
        cmd.prompt.clone()
    } else {
        format!("{}\n\nArgs: {}", cmd.prompt, args)
    };

    let client = crate::ai::AIClient::new();
    let (response, _) = client.generate(&prompt).unwrap_or_else(|e| (e, crate::ai::TokenUsage::default()));

    Ok(response)
}

fn load_commands() -> Result<HashMap<String, CustomCommand>, String> {
    let path = PathBuf::from(COMMANDS_DB);
    if !path.exists() {
        let defaults = default_commands();
        save_commands(&defaults)?;
        return Ok(defaults);
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let commands: Vec<CustomCommand> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for cmd in commands {
        map.insert(cmd.name.clone(), cmd);
    }

    Ok(map)
}

fn save_commands(commands: &HashMap<String, CustomCommand>) -> Result<(), String> {
    let values: Vec<_> = commands.values().collect();
    let content = serde_json::to_string_pretty(&values).map_err(|e| e.to_string())?;

    fs::create_dir_all(".devutils").ok();
    fs::write(COMMANDS_DB, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn default_commands() -> HashMap<String, CustomCommand> {
    let mut commands = HashMap::new();

    commands.insert(
        "test".to_string(),
        CustomCommand::new(
            "test",
            "Run tests with coverage",
            "Run the full test suite with coverage report and show any failures.",
        ),
    );

    commands.insert("review".to_string(), CustomCommand::new(
        "review", "Code review", "Review the recent changes and provide feedback on code quality, potential bugs, and improvements."
    ));

    commands.insert(
        "refactor".to_string(),
        CustomCommand::new(
            "refactor",
            "Refactor code",
            "Analyze the codebase and suggest refactoring opportunities for better code quality.",
        ),
    );

    commands
}

pub fn list_builtin_commands() {
    println!("\n\x1b[36m📝 Custom Commands\x1b[0m");
    println!("\nUsage:");
    println!("  devutils cmd add <name> <description> <prompt>");
    println!("  devutils cmd list");
    println!("  devutils cmd run <name> [args]");
    println!("  devutils cmd remove <name>");
    println!("\nBuilt-in Commands:");
    println!("  /test     - Run tests with coverage");
    println!("  /review  - Code review");
    println!("  /refactor - Refactor suggestions");
    println!("\nCustom commands can use $ARGUMENTS placeholder.");
    println!("\nExamples:");
    println!("  devutils cmd add fix-bug 'Fix a bug' 'Find and fix the bug in $ARGUMENTS'");
    println!("  devutils cmd run fix-bug auth-login");
}
