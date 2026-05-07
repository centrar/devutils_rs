//! Inline Editing - Cursor-style AI suggestions

pub fn inline_suggest(file: &str) -> String {
    let content = std::fs::read_to_string(file).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let last_lines: Vec<&str> = lines.iter().rev().take(50).map(|s| *s).collect();
    let last_50 = last_lines.join("\n");
    
    let client = crate::ai::AIClient::new();
    client.generate_code(&format!(
        "Suggest 3 inline completions for:\n{}\n\nFormat: 1. <code>",
        last_50
    )).unwrap_or_else(|e| e)
}

pub fn tab_complete(file: &str) -> String {
    let content = std::fs::read_to_string(file).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(10);
    let context = lines[start..].join("\n");
    
    let client = crate::ai::AIClient::new();
    client.generate_code(&format!("Tab-complete:\n{}\nReturn next lines:", context)).unwrap_or_else(|e| e)
}

pub fn quick_edit(pattern: &str, replace: &str) -> String {
    let mut matches = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains(pattern) {
                        matches.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    format!("Found {} files with '{}' -> '{}'", matches.len(), pattern, replace)
}