//! Shell Completion Generator
//! Generates completion scripts for Zsh, Bash, Fish, and PowerShell.

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use crate::cli_types::Cli;

pub fn generate_completions(shell: &str) -> Result<String, String> {
    let mut cmd = Cli::command();
    let shell_enum = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => return Err(format!("Unsupported shell: {}", shell)),
    };

    let mut buf = Vec::new();
    generate(shell_enum, &mut cmd, "devutils", &mut buf);
    
    String::from_utf8(buf).map_err(|e| e.to_string())
}
