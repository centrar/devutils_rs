//! Git Merge Conflict Resolution Module
//! 
//! Provides autonomous resolution of Git merge conflicts
//! Uses AI to analyze and resolve conflicts intelligently

use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents a merge conflict in a file
#[derive(Debug, Clone)]
pub struct MergeConflict {
    pub file_path: String,
    pub our_content: String,      // Current branch content
    pub their_content: String,    // Incoming branch content
    pub base_content: String,     // Common ancestor
}

/// Represents a resolved conflict
#[derive(Debug, Clone)]
pub struct ResolvedConflict {
    pub file_path: String,
    pub resolution: String,
    pub strategy: ResolutionStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionStrategy {
    UseOurs,
    UseTheirs,
    UseBoth,
    UseAiGenerated(String),
    Manual,
}

/// Detect if a file has merge conflicts
pub fn has_conflicts(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    let content = fs::read_to_string(path).unwrap_or_default();
    content.contains("<<<<<<<") && content.contains("=======") && content.contains(">>>>>>>")
}

/// Parse all conflicts from a conflicted file
pub fn parse_conflicts(file_path: &str) -> Result<Vec<MergeConflict>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut conflicts = Vec::new();
    let mut lines = content.lines().peekable();
    let mut current_conflict: Option<MergeConflict> = None;
    let mut in_conflict = false;
    let mut conflict_section = String::new();

    
    while let Some(line) = lines.next() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            conflict_section = String::new();
            current_conflict = Some(MergeConflict {
                file_path: file_path.to_string(),
                our_content: String::new(),
                their_content: String::new(),
                base_content: String::new(),
            });

            
            // Extract file path from header if present
            let file_header = line.strip_prefix("<<<<<<<").unwrap_or("").trim();
            if !file_header.is_empty() && !file_header.starts_with("HEAD") {
                // File was specified in conflict marker
            }
        } else if line.starts_with("|||||||") {
            // base section skipped
        } else if line.starts_with("=======") {
            if let Some(ref mut conf) = current_conflict {
                conf.our_content = conflict_section.trim().to_string();
            }
            conflict_section = String::new();
        } else if line.starts_with(">>>>>>>") {
            in_conflict = false;
            if let Some(ref mut conf) = current_conflict {
                conf.their_content = conflict_section.trim().to_string();
                conflicts.push(current_conflict.take().unwrap());
            }
            conflict_section = String::new();
        } else if in_conflict {
            if !conflict_section.is_empty() || !line.is_empty() {
                conflict_section.push_str(line);
                conflict_section.push('\n');
            }
        }
    }
    
    Ok(conflicts)
}

/// Extract conflict markers from diff3 format
fn parse_conflict_with_base(content: &str) -> Vec<MergeConflict> {
    let mut conflicts = Vec::new();
    let mut in_ours = false;
    let mut in_base = false;
    let mut in_theirs = false;
    let mut current_file = String::new();
    let mut ours_content = String::new();
    let mut base_content = String::new();
    let mut theirs_content = String::new();
    
    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_ours = true;
            ours_content.clear();
            current_file = line.strip_prefix("<<<<<<<").unwrap_or("").trim().to_string();
        } else if line.starts_with("|||||||") {
            in_ours = false;
            in_base = true;
        } else if line.starts_with("=======") {
            in_base = false;
            in_theirs = true;
        } else if line.starts_with(">>>>>>>") {
            in_theirs = false;
            conflicts.push(MergeConflict {
                file_path: current_file.clone(),
                our_content: ours_content.trim().to_string(),
                their_content: theirs_content.trim().to_string(),
                base_content: base_content.trim().to_string(),
            });
        } else if in_ours {
            ours_content.push_str(line);
            ours_content.push('\n');
        } else if in_base {
            base_content.push_str(line);
            base_content.push('\n');
        } else if in_theirs {
            theirs_content.push_str(line);
            theirs_content.push('\n');
        }
    }
    
    conflicts
}

/// Auto-resolve a conflict using AI
pub fn resolve_conflict_ai(
    conflict: &MergeConflict,
    ai_client: &crate::ai::AIClient,
) -> Result<ResolvedConflict, String> {
    let prompt = format!(
        r#"You are resolving a Git merge conflict in file: {}

CONTEXT:
This is an AI-assisted merge resolution. Analyze both versions and create a merged result.

=== OUR VERSION (current branch) ===
{}

=== THEIR VERSION (incoming branch) ===
{}

=== BASE VERSION (common ancestor) ===
{}

TASK:
1. Analyze what changed in each version
2. Determine what the merged result should be
3. Output ONLY the merged file content with no explanations
4. Preserve all important changes from both versions
5. If changes are incompatible, make a sensible decision

Output the complete merged file content:"#,
        conflict.file_path,
        conflict.our_content,
        conflict.their_content,
        conflict.base_content
    );
    
    let resolution = ai_client.generate_code(&prompt).map(|(s, _)| s).unwrap_or_else(|e| e);
    
    Ok(ResolvedConflict {
        file_path: conflict.file_path.clone(),
        resolution,
        strategy: ResolutionStrategy::UseAiGenerated("ai".to_string()),
    })
}

/// Auto-resolve using simple strategies
pub fn resolve_conflict_simple(
    conflict: &MergeConflict,
    strategy: &ResolutionStrategy,
) -> ResolvedConflict {
    let resolution = match strategy {
        ResolutionStrategy::UseOurs => conflict.our_content.clone(),
        ResolutionStrategy::UseTheirs => conflict.their_content.clone(),
        ResolutionStrategy::UseBoth => {
            format!("{}\n\n// ======= MERGED =======\n\n{}",
                conflict.our_content, conflict.their_content)
        }
        ResolutionStrategy::UseAiGenerated(content) => content.clone(),
        ResolutionStrategy::Manual => conflict.our_content.clone(),
    };

    ResolvedConflict {
        file_path: conflict.file_path.clone(),
        resolution,
        strategy: strategy.clone(),
    }
}

/// Apply a resolved conflict to a file
pub fn apply_resolution(resolved: &ResolvedConflict) -> Result<(), String> {
    fs::write(&resolved.file_path, &resolved.resolution)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// Resolve all conflicts in a file with AI
pub fn resolve_file_with_ai(
    file_path: &str,
    use_ours_if_ai_fails: bool,
) -> Result<Vec<ResolvedConflict>, String> {
    let conflicts = parse_conflicts(file_path)?;
    
    if conflicts.is_empty() {
        return Ok(vec![]);
    }
    
    let ai_client = crate::ai::AIClient::new();
    let mut resolved = Vec::new();
    
    for conflict in conflicts {
        let result = resolve_conflict_ai(&conflict, &ai_client);
        
        match result {
            Ok(r) => resolved.push(r),
            Err(e) if use_ours_if_ai_fails => {
                resolved.push(ResolvedConflict {
                    file_path: conflict.file_path.clone(),
                    resolution: conflict.our_content.clone(),
                    strategy: ResolutionStrategy::UseOurs,
                });
            }
            Err(e) => return Err(format!("Failed to resolve {}: {}", conflict.file_path, e)),
        }
    }
    
    Ok(resolved)
}

/// Main merge resolution command
pub fn run_merge_resolve(
    _branch: Option<&str>,
    strategy: Option<&str>,
    dry_run: bool,
) -> Result<String, String> {
    // Check for conflicted files
    let conflicted_files = get_conflicted_files()?;
    
    if conflicted_files.is_empty() {
        return Ok("No merge conflicts found".to_string());
    }
    
    let mut output = format!("Found {} files with conflicts:\n", conflicted_files.len());
    
    for file in &conflicted_files {
        output.push_str(&format!("  - {}\n", file));
    }
    
    let resolution_strategy = match strategy {
        Some("ours") => ResolutionStrategy::UseOurs,
        Some("theirs") => ResolutionStrategy::UseTheirs,
        Some("both") => ResolutionStrategy::UseBoth,
        Some("ai") | None => ResolutionStrategy::UseAiGenerated("ai".to_string()),
        Some(s) => return Err(format!("Unknown strategy: {}. Use: ours, theirs, both, ai", s)),
    };
    
    if dry_run {
        output.push_str("\n⚠️  Dry run - no changes will be made\n");
        
        for file in &conflicted_files {
            let conflicts = parse_conflicts(file)?;
            for conflict in conflicts {
                output.push_str(&format!("\n📄 {}:\n", conflict.file_path));
                output.push_str(&format!("  Our lines: {}\n", conflict.our_content.lines().count()));
                output.push_str(&format!("  Their lines: {}\n", conflict.their_content.lines().count()));
            }
        }
        
        return Ok(output);
    }
    
    // Resolve conflicts
    let mut resolved_count = 0;
    
    for file in &conflicted_files {
        let conflicts = parse_conflicts(file)?;
        
        for conflict in conflicts {
            let resolved = match &resolution_strategy {
                ResolutionStrategy::UseAiGenerated(_) => {
                    resolve_conflict_ai(&conflict, &crate::ai::AIClient::new())?
                }
                _ => resolve_conflict_simple(&conflict, &resolution_strategy),
            };
            
            apply_resolution(&resolved)?;
            resolved_count += 1;
            
            output.push_str(&format!(
                "✅ Resolved {} using {:?}\n",
                resolved.file_path, resolved.strategy
            ));
        }
    }
    
    output.push_str(&format!("\n✨ Resolved {} conflicts total\n", resolved_count));
    
    Ok(output)
}

/// Get list of all conflicted files in git
pub fn get_conflicted_files() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    
    if !output.status.success() {
        return Err("Git command failed".to_string());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = stdout
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    Ok(files)
}

/// Abort a merge and reset to pre-merge state
pub fn abort_merge() -> Result<String, String> {
    let output = Command::new("git")
        .args(["merge", "--abort"])
        .output()
        .map_err(|e| format!("Failed to run git merge --abort: {}", e))?;
    
    if output.status.success() {
        Ok("Merge aborted successfully".to_string())
    } else {
        Err(format!("Failed to abort merge: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

/// Continue a merge after resolving conflicts
pub fn continue_merge() -> Result<String, String> {
    // First stage resolved files
    let conflicted_files = get_conflicted_files()?;
    
    for file in &conflicted_files {
        Command::new("git")
            .args(["add", file])
            .output()
            .map_err(|e| format!("Failed to stage file: {}", e))?;
    }
    
    let output = Command::new("git")
        .args(["commit", "--no-edit"])
        .output()
        .map_err(|e| format!("Failed to commit merge: {}", e))?;
    
    if output.status.success() {
        Ok("Merge completed successfully".to_string())
    } else {
        Err(format!("Failed to complete merge: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_conflicts() {
        let temp_file = std::env::temp_dir().join("test_conflict.txt");
        fs::write(&temp_file, "<<<<<<< HEAD\ncontent a\n=======\ncontent b\n>>>>>>> branch\n").unwrap();
        
        assert!(has_conflicts(&temp_file));
        
        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_no_conflicts() {
        let temp_file = std::env::temp_dir().join("test_no_conflict.txt");
        fs::write(&temp_file, "normal content\n").unwrap();
        
        assert!(!has_conflicts(&temp_file));
        
        fs::remove_file(&temp_file).ok();
    }
}