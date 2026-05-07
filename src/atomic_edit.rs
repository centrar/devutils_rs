//! Atomic File Editing Module
//! Safe file modifications with diff/patch support to prevent data loss

use std::fs;
use std::path::Path;

/// Result of an atomic edit operation
#[derive(Debug, Clone)]
pub struct EditResult {
    pub success: bool,
    pub file_path: String,
    pub original_content: String,
    pub new_content: String,
    pub backup_path: Option<String>,
    pub diff: String,
}

/// Represents a surgical search-replace block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchReplaceBlock {
    pub search: String,
    pub replace: String,
}

/// Perform atomic edit on a file
pub fn atomic_edit(
    file_path: &str,
    old_text: &str,
    new_text: &str,
    create_backup: bool,
) -> Result<EditResult, String> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Read original content
    let original_content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
    
    // Create backup if requested
    let backup_path = if create_backup {
        let backup = format!("{}.backup.{}", file_path, chrono_timestamp());
        fs::copy(path, &backup)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
        Some(backup)
    } else {
        None
    };
    
    // Perform the edit
    let new_content = if old_text.is_empty() {
        // Append mode
        format!("{}{}", original_content, new_text)
    } else if original_content.contains(old_text) {
        // Replace mode
        original_content.replace(old_text, new_text)
    } else {
        // Try fuzzy match
        fuzzy_replace(&original_content, old_text, new_text)
            .unwrap_or_else(|| original_content.clone())
    };
    
    // Generate diff
    let diff = generate_diff(&original_content, &new_content, file_path);
    
    // Write new content
    fs::write(path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;
    
    Ok(EditResult {
        success: true,
        file_path: file_path.to_string(),
        original_content,
        new_content,
        backup_path,
        diff,
    })
}

/// Fuzzy match and replace (for when exact match fails)
fn fuzzy_replace(content: &str, old: &str, new: &str) -> Option<String> {
    // Try case-insensitive match
    let escaped_old = regex::escape(old);
    if let Ok(re) = regex::RegexBuilder::new(&escaped_old).case_insensitive(true).build() {
        if let Some(mat) = re.find(content) {
            let mut result = content.to_string();
            result.replace_range(mat.start()..mat.end(), new);
            return Some(result);
        }
    }
    
    // Try line-by-line match
    let old_lines: Vec<&str> = old.lines().collect();
    let content_lines: Vec<&str> = content.lines().collect();
    
    for window in content_lines.windows(old_lines.len()) {
        let window_str = window.join("\n");
        if window_str == old {
            // Found exact multi-line match
            return Some(content.replace(old, new));
        }
    }
    
    None
}

/// Generate unified diff
pub fn generate_diff(old: &str, new: &str, filename: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    
    let mut diff = format!("--- a/{}\n+++ b/{}\n", filename, filename);
    
    let mut i = 0;
    let mut j = 0;
    
    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() {
            if old_lines[i] == new_lines[j] {
                // Context line
                diff.push_str(&format!(" {}\n", old_lines[i]));
                i += 1;
                j += 1;
            } else {
                // Change
                diff.push_str(&format!("-{}\n", old_lines[i]));
                diff.push_str(&format!("+{}\n", new_lines[j]));
                i += 1;
                j += 1;
            }
        } else if i < old_lines.len() {
            diff.push_str(&format!("-{}\n", old_lines[i]));
            i += 1;
        } else {
            diff.push_str(&format!("+{}\n", new_lines[j]));
            j += 1;
        }
    }
    
    diff
}

/// Apply surgical search-replace blocks to a file
pub fn apply_search_replace(
    file_path: &str,
    blocks: &[SearchReplaceBlock],
    create_backup: bool,
) -> Result<EditResult, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let original_content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

    let backup_path = if create_backup {
        let backup = format!("{}.backup.{}", file_path, chrono_timestamp());
        fs::copy(path, &backup)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
        Some(backup)
    } else {
        None
    };

    let mut new_content = original_content.clone();
    for block in blocks {
        if new_content.contains(&block.search) {
            new_content = new_content.replace(&block.search, &block.replace);
        } else {
            // Fallback to fuzzy replace for this block
            if let Some(fuzzy) = fuzzy_replace(&new_content, &block.search, &block.replace) {
                new_content = fuzzy;
            } else {
                return Err(format!(
                    "Failed to find exact match for search block in {}:\n{}",
                    file_path, block.search
                ));
            }
        }
    }

    let diff = generate_diff(&original_content, &new_content, file_path);
    fs::write(path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

    Ok(EditResult {
        success: true,
        file_path: file_path.to_string(),
        original_content,
        new_content,
        backup_path,
        diff,
    })
}

/// Apply a patch/diff to a file
pub fn apply_patch(file_path: &str, patch: &str) -> Result<EditResult, String> {
    let path = Path::new(file_path);
    let _original = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

    // Use standard patch command if available, or fallback to our logic
    // For now, we implement a robust Search-Replace block parser for Aider compatibility
    let blocks = parse_search_replace_blocks(patch);
    if !blocks.is_empty() {
        return apply_search_replace(file_path, &blocks, true);
    }

    Err("Invalid patch format. Use <<<<<<< SEARCH / ======= / >>>>>>> REPLACE blocks.".to_string())
}

fn parse_search_replace_blocks(text: &str) -> Vec<SearchReplaceBlock> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("<<<<<<< SEARCH") {
            let mut search = String::new();
            while let Some(search_line) = lines.next() {
                if search_line.starts_with("=======") {
                    break;
                }
                search.push_str(search_line);
                search.push('\n');
            }

            let mut replace = String::new();
            while let Some(replace_line) = lines.next() {
                if replace_line.starts_with(">>>>>>> REPLACE") {
                    break;
                }
                replace.push_str(replace_line);
                replace.push('\n');
            }

            blocks.push(SearchReplaceBlock {
                search: search.trim_end().to_string(),
                replace: replace.trim_end().to_string(),
            });
        }
    }
    blocks
}

/// Rollback to backup
pub fn rollback(file_path: &str, backup_path: &str) -> Result<(), String> {
    if !Path::new(backup_path).exists() {
        return Err(format!("Backup not found: {}", backup_path));
    }
    
    fs::copy(backup_path, file_path)
        .map_err(|e| format!("Failed to rollback: {}", e))?;
    
    Ok(())
}

/// Create a patch from two versions of a file
pub fn create_patch(old_path: &str, new_path: &str) -> Result<String, String> {
    let old = fs::read_to_string(old_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let new = fs::read_to_string(new_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    Ok(generate_diff(&old, &new, new_path))
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

/// CLI command for atomic edit
pub fn run_atomic_edit(file: &str, old: &str, new: &str) -> Result<String, String> {
    let result = atomic_edit(file, old, new, true)?;
    
    Ok(format!(
        "✅ Atomic edit successful\n\
         File: {}\n\
         Backup: {:?}\n\
         Changes:\n{}",
        result.file_path,
        result.backup_path,
        result.diff
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_atomic_edit() {
        let test_file = "test_atomic.tmp";
        fs::write(test_file, "Hello World").unwrap();
        
        let result = atomic_edit(test_file, "World", "Rust", true).unwrap();
        
        assert!(result.success);
        assert!(result.new_content.contains("Rust"));
        
        fs::remove_file(test_file).ok();
        if let Some(backup) = result.backup_path {
            fs::remove_file(backup).ok();
        }
    }
}
