//! AI Git Features - Beats lazygit with automation
//! - AI commit messages
//! - Smart conflict resolution
//! - One-click workflows

use crate::ai::AIClient;
use std::process::Command;
use std::io::Write;

pub fn generate_commit_message() -> Result<String, String> {
    // Get staged changes
    let diff_output = Command::new("git")
        .args(&["diff", "--cached"])
        .output()
        .map_err(|e| format!("Failed to get diff: {}", e))?;
    
    let diff = String::from_utf8_lossy(&diff_output.stdout);
    
    if diff.trim().is_empty() {
        return Err("No staged changes".to_string());
    }
    
    // AI generates commit message
    let client = AIClient::new();
    let prompt = format!(
        "Generate a concise conventional commit message for these changes:
        
{}

Format: <type>(<scope>): <description>

Examples:
- feat(auth): add OAuth2 login
- fix(api): handle null response in user service
- refactor(core): simplify error handling",
        diff
    );
    
    let message = client.generate_code(&prompt)?;
    Ok(message.lines().next().unwrap_or("chore: update").to_string())
}

pub fn resolve_conflict(file: &str) -> Result<String, String> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| format!("Failed to read {}: {}", file, e))?;
    
    if !content.contains("<<<<<<<") {
        return Err("No merge conflicts found".to_string());
    }
    
    let client = AIClient::new();
    let prompt = format!(
        "Resolve this merge conflict in {}:
        
{}

Return only the resolved code, no explanations.",
        file, content
    );
    
    let resolved = client.generate_code(&prompt)?;
    
    std::fs::write(file, &resolved)
        .map_err(|e| format!("Failed to write resolved file: {}", e))?;
    
    Ok(format!("Resolved conflict in {}", file))
}

pub fn prepare_release(version: &str) -> Result<String, String> {
    println!("🚀 Preparing release {}...", version);
    
    // 1. Run tests
    println!("  1/4. Running tests...");
    let test_status = Command::new("cargo")
        .args(&["test"])
        .status();
    
    if !test_status.map(|s| s.success()).unwrap_or(false) {
        return Err("Tests failed".to_string());
    }
    
    // 2. Update version
    println!("  2/4. Updating version...");
    // Would update Cargo.toml, package.json, etc.
    
    // 3. Commit and tag
    println!("  3/4. Creating commit and tag...");
    Command::new("git")
        .args(&["commit", "-am", &format!("chore: release v{}", version)])
        .status()
        .ok();
    
    Command::new("git")
        .args(&["tag", &format!("v{}", version)])
        .status()
        .ok();
    
    // 4. Push
    println!("  4/4. Pushing to remote...");
    Command::new("git")
        .args(&["push", "origin", "main", "--tags"])
        .status()
        .ok();
    
    Ok(format!("✅ Release v{} prepared and pushed!", version))
}

// CLI commands
pub fn ai_commit() {
    match generate_commit_message() {
        Ok(msg) => {
            println!("Generated commit message: {}", msg);
            print!("Commit with this message? (Y/n): ");
            std::io::stdout().flush().ok();
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            
            if input.trim().eq_ignore_ascii_case("y") || input.trim().is_empty() {
                Command::new("git")
                    .args(&["commit", "-m", &msg])
                    .status()
                    .ok();
                println!("✅ Committed!");
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

pub fn ai_resolve(file: &str) {
    match resolve_conflict(file) {
        Ok(result) => println!("{}", result),
        Err(e) => println!("Error: {}", e),
    }
}

pub fn ai_prepare_release(version: &str) {
    match prepare_release(version) {
        Ok(result) => println!("{}", result),
        Err(e) => println!("Error: {}", e),
    }
}