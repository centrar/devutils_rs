//! Git Module

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct GitOpts {
    #[command(subcommand)]
    pub command: GitCommands,
}

#[derive(Subcommand)]
pub enum GitCommands {
    Status,
    Commits {
        #[arg(default_value_t = 10)]
        n: usize,
    },
    Branches,
    Branch,
    Commit {
        message: String,
    },
    CommitAuto,
    Push,
    Ignore {
        template: Option<String>,
    },
    Diff {
        #[arg(short, long)]
        staged: bool,
    },
    SessionStart,
    SessionRollback,
}

pub struct GitOps;

impl GitOps {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, cmd: GitCommands) -> Result<()> {
        use std::process::Command;
        match cmd {
            GitCommands::Status => {
                let o = Command::new("git").args(["status", "--short"]).output()?;
                println!("\n📊 Git Status");
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    if s.is_empty() {
                        println!("  ✅ Clean");
                    } else {
                        for l in s.lines().take(10) {
                            println!("  {}", l);
                        }
                    }
                }
            }
            GitCommands::Commits { n } => {
                let o = Command::new("git")
                    .args(["log", &format!("-{}", n), "--oneline"])
                    .output()?;
                println!("\n📜 Commits:");
                for l in String::from_utf8_lossy(&o.stdout).lines().take(10) {
                    println!("  {}", l);
                }
            }
            GitCommands::Branches => {
                let o = Command::new("git").args(["branch", "-a"]).output()?;
                for l in String::from_utf8_lossy(&o.stdout).lines() {
                    println!("  {}", l.trim());
                }
            }
            GitCommands::Branch => {
                let o = Command::new("git")
                    .args(["branch", "--show-current"])
                    .output()?;
                println!("  📍 {}", String::from_utf8_lossy(&o.stdout).trim());
            }
            GitCommands::Commit { message } => {
                Command::new("git").args(["add", "-A"]).output()?;
                Command::new("git")
                    .args(["commit", "-m", &message])
                    .output()?;
                println!("  ✅ Committed: {}", message);
            }
            GitCommands::Push => {
                let o = Command::new("git").args(["push"]).output()?;
                if o.status.success() {
                    println!("  ⬆️ Pushed");
                } else {
                    println!("  ❌ Failed");
                }
            }
            GitCommands::Ignore { template } => {
                let content = match template.as_deref() {
                    Some("python") => "# Python\n__pycache__/\n*.pyc\nvenv/\n",
                    Some("node") => "# Node\nnode_modules/\n*.log\n",
                    _ => "# DevUtils\nnode_modules/\n__pycache__/\n",
                };
                std::fs::write(".gitignore", content)?;
                println!("  ✅ .gitignore created");
            }
            GitCommands::Diff { staged } => {
                let o = if staged {
                    Command::new("git").args(["diff", "--cached"]).output()?
                } else {
                    Command::new("git").arg("diff").output()?
                };
                print!("{}", String::from_utf8_lossy(&o.stdout));
            }
            GitCommands::CommitAuto => {
                let diff = Command::new("git").args(["diff", "--cached"]).output()?;
                let diff_str = String::from_utf8_lossy(&diff.stdout);
                if diff_str.is_empty() {
                    println!("  ⚠️ No staged changes to commit.");
                    return Ok(());
                }

                use crate::ai::AIClient;
                let client = AIClient::new();
                let prompt = format!(
                    "Generate a concise, professional Git commit message for these changes:\n\n{}",
                    diff_str
                );
                
                let (message, _) = client.generate_code(&prompt).unwrap_or_else(|_| ("update: automated commit".to_string(), crate::ai::TokenUsage::default()));
                let clean_msg = message.lines().next().unwrap_or("update: automated commit").trim_matches(|c| c == '`' || c == '"');
                
                Command::new("git")
                    .args(["commit", "-m", clean_msg])
                    .output()?;
                println!("  ✅ Auto-Committed: {}", clean_msg);
            }
            GitCommands::SessionStart => {
                // Create a temporary "devutils-session" tag or branch
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let tag = format!("devutils_session_{}", timestamp);
                Command::new("git").args(["tag", &tag]).output()?;
                println!("  🏁 Session started. Tag: {}", tag);
            }
            GitCommands::SessionRollback => {
                // Find the latest devutils tag and reset to it
                let output = Command::new("git")
                    .args(["tag", "--list", "devutils_session_*", "--sort=-creatordate"])
                    .output()?;
                let tags = String::from_utf8_lossy(&output.stdout);
                if let Some(latest) = tags.lines().next() {
                    println!("  ⏪ Rolling back to: {}", latest);
                    Command::new("git").args(["reset", "--hard", latest]).output()?;
                } else {
                    println!("  ⚠️ No devutils sessions found to rollback.");
                }
            }
        }
        Ok(())
    }
}
