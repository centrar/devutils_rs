use std::fs;
use std::path::Path;
use std::process::Command;
use std::env;

pub struct ScriptRunner {
    args: Vec<String>,
}

impl ScriptRunner {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    pub fn run_script(&self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read script '{}': {}", path, e))?;

        let mut line_num = 0;
        for line in content.lines() {
            line_num += 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Expand variables and run command
            let expanded = self.expand_variables(trimmed);
            if let Err(e) = self.run_command(&expanded) {
                return Err(format!("Error on line {}: {}", line_num, e));
            }
        }

        Ok(())
    }

    fn expand_variables(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Expand $1, $2, etc.
        for (i, arg) in self.args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Expand environment variables
        for (key, value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &value);
        }

        result
    }

    fn run_command(&self, cmd: &str) -> Result<(), String> {
        // Skip error-suppressed commands (prefixed with -)
        let cmd = cmd.strip_prefix('-').unwrap_or(cmd).trim();
        if cmd.is_empty() {
            return Ok(());
        }

        // Find the devutils executable
        let exe = env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("devutils"));

        // Execute devutils with the command
        let output = Command::new(&exe)
            .args(cmd.split_whitespace())
            .output()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        // Print output
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        // Check exit code
        if !output.status.success() {
            return Err(format!("Command failed with exit code: {:?}", output.status.code()));
        }

        Ok(())
    }
}

pub fn run_script_file(path: &str, args: Vec<String>) -> Result<(), String> {
    let runner = ScriptRunner::new(args);
    runner.run_script(path)
}