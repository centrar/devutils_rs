//! Skills System - Reusable capability modules
//!
//! Skills are reusable knowledge modules that extend AI abilities:
//! - Custom slash commands
//! - Prompt templates
//! - Specialized workflows
//! - Domain-specific knowledge
//!
//! Skills are stored in .devutils/skills/ directory

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::RwLock;

static SKILLS: Lazy<RwLock<HashMap<String, Skill>>> = Lazy::new(|| RwLock::new(default_skills()));

fn default_skills() -> HashMap<String, Skill> {
    let mut skills = HashMap::new();

    skills.insert(
        "explain".to_string(),
        Skill {
            name: "explain".to_string(),
            description: "Explain code in detail".to_string(),
            prompt: "Explain this code in detail, including its purpose, logic, and how it works."
                .to_string(),
            category: "analysis".to_string(),
            tags: vec!["explain".to_string(), "documentation".to_string()],
        },
    );

    skills.insert(
        "refactor".to_string(),
        Skill {
            name: "refactor".to_string(),
            description: "Refactor code for readability".to_string(),
            prompt: "Refactor this code to be more readable while maintaining functionality."
                .to_string(),
            category: "refactoring".to_string(),
            tags: vec!["refactor".to_string(), "cleanup".to_string()],
        },
    );

    skills.insert(
        "test".to_string(),
        Skill {
            name: "test".to_string(),
            description: "Generate tests".to_string(),
            prompt: "Generate comprehensive tests for this code, covering edge cases.".to_string(),
            category: "testing".to_string(),
            tags: vec!["test".to_string(), "coverage".to_string()],
        },
    );

    skills.insert(
        "debug".to_string(),
        Skill {
            name: "debug".to_string(),
            description: "Debug issues".to_string(),
            prompt: "Debug this code and identify the root cause of any issues.".to_string(),
            category: "debugging".to_string(),
            tags: vec!["debug".to_string(), "fix".to_string()],
        },
    );

    skills.insert(
        "security".to_string(),
        Skill {
            name: "security".to_string(),
            description: "Security audit".to_string(),
            prompt: "Audit this code for security vulnerabilities.".to_string(),
            category: "security".to_string(),
            tags: vec!["security".to_string(), "audit".to_string()],
        },
    );

    skills.insert(
        "optimize".to_string(),
        Skill {
            name: "optimize".to_string(),
            description: "Optimize performance".to_string(),
            prompt: "Optimize this code for better performance.".to_string(),
            category: "optimization".to_string(),
            tags: vec!["optimize".to_string(), "performance".to_string()],
        },
    );

    skills.insert(
        "document".to_string(),
        Skill {
            name: "document".to_string(),
            description: "Generate documentation".to_string(),
            prompt: "Generate documentation for this code.".to_string(),
            category: "documentation".to_string(),
            tags: vec!["docs".to_string(), "documentation".to_string()],
        },
    );

    skills.insert(
        "review".to_string(),
        Skill {
            name: "review".to_string(),
            description: "Code review".to_string(),
            prompt: "Perform a thorough code review, checking for bugs, style, and best practices."
                .to_string(),
            category: "review".to_string(),
            tags: vec!["review".to_string(), "quality".to_string()],
        },
    );

    skills
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub category: String,
    pub tags: Vec<String>,
}

pub fn list_skills() -> String {
    let skills = SKILLS.read().unwrap();
    let list: Vec<&Skill> = skills.values().collect();
    serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
}

pub fn get_skill(name: &str) -> String {
    let skills = SKILLS.read().unwrap();
    skills
        .get(name)
        .map(|s| s.prompt.clone())
        .unwrap_or_else(|| "".to_string())
}

pub fn search_skills(query: &str) -> String {
    let q = query.to_lowercase();
    let skills = SKILLS.read().unwrap();
    let results: Vec<&Skill> = skills
        .values()
        .filter(|s| {
            s.name.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
                || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect();
    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
}

pub fn add_skill(name: &str, prompt: &str) -> String {
    let mut skills = SKILLS.write().unwrap();
    skills.insert(
        name.to_string(),
        Skill {
            name: name.to_string(),
            description: format!("Custom skill: {}", name),
            prompt: prompt.to_string(),
            category: "custom".to_string(),
            tags: vec![],
        },
    );
    format!("Skill '{}' added", name)
}

pub fn remove_skill(name: &str) -> String {
    let mut skills = SKILLS.write().unwrap();
    if skills.remove(name).is_some() {
        format!("Skill '{}' removed", name)
    } else {
        format!("Skill not found: {}", name)
    }
}

pub fn skills_count() -> usize {
    let skills = SKILLS.read().unwrap();
    skills.len()
}

pub fn get_skill_prompt(name: &str) -> String {
    let skills = SKILLS.read().unwrap();
    skills
        .get(name)
        .map(|s| s.prompt.clone())
        .unwrap_or_else(|| "".to_string())
}

pub fn skills_commands() {
    println!("\n\x1b[36m🎯 Skills System:\x1b[0m\n");
    println!("  \x1b[33mList:\x1b[0m   devutils skills list");
    println!("  \x1b[33mSearch:\x1b[0m  devutils skills search <query>");
    println!("  \x1b[33mGet:\x1b[0m   devutils skills get <name>");
    println!("  \x1b[33mAdd:\x1b[0m   devutils skills add <name> <description> <prompt>");
    println!("  \x1b[33mRemove:\x1b[0m devutils skills remove <name>");
    println!();
}
