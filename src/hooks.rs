//! Hooks System - Automation triggered by events

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::RwLock;

static HOOKS: Lazy<RwLock<HashMap<String, HookConfig>>> =
    Lazy::new(|| RwLock::new(default_hooks()));

fn default_hooks() -> HashMap<String, HookConfig> {
    let mut hooks = HashMap::new();

    hooks.insert(
        "pre-commit".to_string(),
        HookConfig {
            name: "pre-commit".to_string(),
            description: "Run before git commit".to_string(),
            command: "cargo fmt --check".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks.insert(
        "post-commit".to_string(),
        HookConfig {
            name: "post-commit".to_string(),
            description: "Run after git commit".to_string(),
            command: "echo 'Committed!'".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks.insert(
        "pre-build".to_string(),
        HookConfig {
            name: "pre-build".to_string(),
            description: "Run before build".to_string(),
            command: "cargo check".to_string(),
            enabled: false,
            allow_fail: true,
        },
    );

    hooks.insert(
        "post-build".to_string(),
        HookConfig {
            name: "post-build".to_string(),
            description: "Run after build".to_string(),
            command: "echo 'Build complete!'".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks.insert(
        "pre-test".to_string(),
        HookConfig {
            name: "pre-test".to_string(),
            description: "Run before tests".to_string(),
            command: "cargo check".to_string(),
            enabled: false,
            allow_fail: true,
        },
    );

    hooks.insert(
        "post-test".to_string(),
        HookConfig {
            name: "post-test".to_string(),
            description: "Run after tests".to_string(),
            command: "echo 'Tests done!'".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks.insert(
        "on-save".to_string(),
        HookConfig {
            name: "on-save".to_string(),
            description: "Run when files are saved".to_string(),
            command: "".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks.insert(
        "on-error".to_string(),
        HookConfig {
            name: "on-error".to_string(),
            description: "Run when errors occur".to_string(),
            command: "echo 'Error occurred'".to_string(),
            enabled: false,
            allow_fail: false,
        },
    );

    hooks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub name: String,
    pub description: String,
    pub command: String,
    pub enabled: bool,
    pub allow_fail: bool,
}

pub fn list_hooks() -> String {
    let hooks = HOOKS.read().unwrap();
    let mut result = String::new();
    for hook in hooks.values() {
        result.push_str(&format!(
            "{}: {} [{}] command: {}\n",
            hook.name,
            hook.description,
            if hook.enabled { "enabled" } else { "disabled" },
            if hook.command.is_empty() {
                "(none)"
            } else {
                &hook.command
            }
        ));
    }
    result
}

pub fn get_hook(name: &str) -> String {
    let hooks = HOOKS.read().unwrap();
    match hooks.get(name) {
        Some(hook) => format!(
            "{}: {} [{}] command: {}",
            hook.name,
            hook.description,
            if hook.enabled { "enabled" } else { "disabled" },
            if hook.command.is_empty() {
                "(none)"
            } else {
                &hook.command
            }
        ),
        None => format!("Hook not found: {}", name),
    }
}

pub fn enable_hook(name: &str) -> String {
    let mut hooks = HOOKS.write().unwrap();
    match hooks.get_mut(name) {
        Some(hook) => {
            hook.enabled = true;
            format!("Hook '{}' enabled", name)
        }
        None => format!("Hook not found: {}", name),
    }
}

pub fn disable_hook(name: &str) -> String {
    let mut hooks = HOOKS.write().unwrap();
    match hooks.get_mut(name) {
        Some(hook) => {
            hook.enabled = false;
            format!("Hook '{}' disabled", name)
        }
        None => format!("Hook not found: {}", name),
    }
}

pub fn set_hook_command(name: &str, cmd: &str) -> String {
    let mut hooks = HOOKS.write().unwrap();
    match hooks.get_mut(name) {
        Some(hook) => {
            hook.command = cmd.to_string();
            format!("Hook '{}' command set to: {}", name, cmd)
        }
        None => format!("Hook not found: {}", name),
    }
}

pub fn run_hook(name: &str) -> String {
    let hooks = HOOKS.read().unwrap();
    match hooks.get(name) {
        Some(hook) => {
            if !hook.enabled {
                return format!("Hook '{}' is disabled", name);
            }
            if hook.command.is_empty() {
                return format!("Hook '{}' has no command", name);
            }

            let output = Command::new("sh").arg("-c").arg(&hook.command).output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    if out.status.success() {
                        stdout
                    } else if hook.allow_fail {
                        format!("Warning: {} {}", stdout, stderr)
                    } else {
                        format!("Hook '{}' failed: {}", name, stderr)
                    }
                }
                Err(e) => format!("Hook '{}' error: {}", name, e),
            }
        }
        None => format!("Hook not found: {}", name),
    }
}

pub fn run_enabled_hooks(event: &str) -> Vec<(String, bool, String)> {
    let mut results = vec![];
    let hooks = HOOKS.read().unwrap();

    if let Some(hook) = hooks.get(event) {
        if hook.enabled && !hook.command.is_empty() {
            let output = Command::new("sh").arg("-c").arg(&hook.command).output();

            match output {
                Ok(out) => {
                    let success = out.status.success();
                    let output = String::from_utf8_lossy(&out.stdout).to_string();
                    results.push((event.to_string(), success, output));
                }
                Err(e) => {
                    results.push((event.to_string(), false, e.to_string()));
                }
            }
        }
    }

    results
}

pub fn hooks_count() -> usize {
    let hooks = HOOKS.read().unwrap();
    hooks.len()
}

pub fn hooks_commands() {
    println!("\n\x1b[36m🪝 Hooks System:\x1b[0m\n");
    println!("  \x1b[33mList:\x1b[0m   devutils hooks list");
    println!("  \x1b[33mEnable:\x1b[0m  devutils hooks enable <name>");
    println!("  \x1b[33mDisable:\x1b[0m devutils hooks disable <name>");
    println!("  \x1b[33mSet:\x1b[0m   devutils hooks set <name> <command>");
    println!("  \x1b[33mRun:\x1b[0m   devutils hooks run <name>");
    println!();
}
