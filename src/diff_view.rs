//! Diff Viewer - Terminal-based diff viewer with syntax highlighting

use std::fmt;
use std::process::Command;

pub struct DiffViewer {
    pub show_context: usize,
    pub color: bool,
    pub word_diff: bool,
}

impl DiffViewer {
    pub fn new() -> Self {
        Self {
            show_context: 3,
            color: true,
            word_diff: false,
        }
    }

    pub fn show_staged(&self) -> Result<String, String> {
        let output = Command::new("git")
            .args(["diff", "--staged", "--stat"])
            .output()
            .map_err(|e| e.to_string())?;

        let stats = String::from_utf8_lossy(&output.stdout);

        let output = Command::new("git")
            .args(["diff", "--staged", "--numstat"])
            .output()
            .map_err(|e| e.to_string())?;

        let numstat = String::from_utf8_lossy(&output.stdout);

        let mut result = String::new();
        result.push_str("\n\x1b[36m📊 Staged Changes\x1b[0m\n\n");
        result.push_str(&format!("{}\n\n", stats));

        for line in numstat.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let add = parts[0].parse::<i64>().unwrap_or(0);
                let del = parts[1].parse::<i64>().unwrap_or(0);
                let file = parts[2];

                let color = if add > 0 && del > 0 {
                    "\x1b[33m"
                } else if add > 0 {
                    "\x1b[32m"
                } else if del > 0 {
                    "\x1b[31m"
                } else {
                    "\x1b[90m"
                };

                result.push_str(&format!("{} +{} -{}\x1b[0m {} \n", color, add, del, file));
            }
        }

        Ok(result)
    }

    pub fn show_modified(&self) -> Result<String, String> {
        let output = Command::new("git")
            .args(["diff", "--name-status"])
            .output()
            .map_err(|e| e.to_string())?;

        let status = String::from_utf8_lossy(&output.stdout);

        let mut result = String::new();
        result.push_str("\n\x1b[36m📝 Unstaged Changes\x1b[0m\n\n");

        for line in status.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let op = parts[0];
                let file = parts[1];

                let symbol = match op {
                    "M" => "\x1b[33mM\x1b[0m",
                    "A" => "\x1b[32mA\x1b[0m",
                    "D" => "\x1b[31mD\x1b[0m",
                    "R" => "\x1b[36mR\x1b[0m",
                    _ => "\x1b[90m?\x1b[0m",
                };

                result.push_str(&format!("{} {}\n", symbol, file));
            }
        }

        Ok(result)
    }

    pub fn show_file_diff(&self, file: &str) -> Result<String, String> {
        let is_staged = file.starts_with("s:");
        let actual_file = if is_staged { &file[2..] } else { file };

        let args = if is_staged {
            vec!["diff", "--staged", "--", actual_file]
        } else {
            vec!["diff", "--", actual_file]
        };

        let output = Command::new("git")
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;

        let diff = String::from_utf8_lossy(&output.stdout);

        if diff.is_empty() {
            return Ok(format!("No changes in {}\n", file));
        }

        let mut result = String::new();
        result.push_str(&format!("\n\x1b[36m📄 {}\x1b[0m\n", file));

        for line in diff.lines() {
            if line.starts_with("@@") {
                result.push_str(&format!("\x1b[35m{}\x1b[0m\n", line));
            } else if line.starts_with('+') && !line.starts_with("+++") {
                result.push_str(&format!("\x1b[32m{}\x1b[0m\n", line));
            } else if line.starts_with('-') && !line.starts_with("---") {
                result.push_str(&format!("\x1b[31m{}\x1b[0m\n", line));
            } else if line.starts_with("diff ") || line.starts_with("index ") {
                result.push_str(&format!("\x1b[90m{}\x1b[0m\n", line));
            } else {
                result.push_str(&format!("{}\n", line));
            }
        }

        Ok(result)
    }

    pub fn show_branch(&self) -> Result<String, String> {
        let current = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map_err(|e| e.to_string())?;

        let branches = Command::new("git")
            .args(["branch", "-a"])
            .output()
            .map_err(|e| e.to_string())?;

        let _current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();
        let all_branches = String::from_utf8_lossy(&branches.stdout);

        let mut result = String::new();
        result.push_str("\n\x1b[36m🌿 Branches\x1b[0m\n\n");

        for line in all_branches.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('*') {
                result.push_str(&format!("\x1b[32m{}\x1b[0m (current)\n", trimmed));
            } else if !trimmed.is_empty() {
                result.push_str(&format!("  {}\n", trimmed));
            }
        }

        Ok(result)
    }

    pub fn show_log(&self, count: usize) -> Result<String, String> {
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", count),
                "--oneline",
                "--graph",
                "--decorate",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let log = String::from_utf8_lossy(&output.stdout);

        let mut result = String::new();
        result.push_str(&format!("\n\x1b[36m📜 Recent {} Commits\x1b[0m\n\n", count));

        for line in log.lines() {
            result.push_str(&format!("{}\n", line));
        }

        Ok(result)
    }
}

impl fmt::Display for DiffViewer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiffViewer")
    }
}

pub fn show_diff_summary() -> Result<String, String> {
    let viewer = DiffViewer::new();

    let staged = viewer.show_staged()?;
    let unstaged = viewer.show_modified()?;

    let mut result = String::new();
    result.push_str(&staged);
    result.push_str("\n");
    result.push_str(&unstaged);

    Ok(result)
}

pub fn diff_commands() {
    println!("\n\x1b[36m📊 Diff Viewer\x1b[0m");
    println!("\nUsage:");
    println!("  devutils diff staged");
    println!("  devutils diff unstaged");
    println!("  devutils diff file <path>");
    println!("  devutils diff branch");
    println!("  devutils diff log [count]");
    println!("  devutils diff summary");
}
