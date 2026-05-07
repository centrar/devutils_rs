//! Shell Interpreter - Autonomous command execution and observation
//! Enables the agent to run scripts, interact with the OS, and debug in real-time.

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc;

pub struct ShellInterpreter {
    pub shell: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration: Duration,
}

impl ShellInterpreter {
    pub fn new() -> Self {
        let shell = if cfg!(target_os = "windows") {
            "powershell"
        } else {
            "bash"
        };
        Self { shell: shell.to_string() }
    }

    /// Translate unix-style commands to windows-style if necessary
    fn translate_command(&self, command: &str) -> String {
        if cfg!(target_os = "windows") {
            command.replace("ls ", "dir ")
                  .replace("rm -rf ", "Remove-Item -Recurse -Force ")
                  .replace("rm ", "del ")
                  .replace("cp ", "copy ")
                  .replace("mv ", "move ")
                  .replace("cat ", "Get-Content ")
                  .replace("mkdir -p ", "New-Item -ItemType Directory -Force ")
        } else {
            command.to_string()
        }
    }

    /// Execute a command and stream output back
    pub fn execute(&self, command: &str, timeout_secs: u64) -> Result<ExecutionResult, String> {
        let translated = self.translate_command(command);
        let start = Instant::now();
        
        let mut child = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(["-Command", &translated])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?
        } else {
            Command::new("bash")
                .args(["-c", &translated])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?
        };

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let (tx, rx) = mpsc::channel();
        let stdout_tx = tx.clone();
        let stderr_tx = tx.clone();

        // Stream stdout
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().filter_map(|l| l.ok()) {
                let _ = stdout_tx.send(format!("OUT: {}", line));
            }
        });

        // Stream stderr
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().filter_map(|l| l.ok()) {
                let _ = stderr_tx.send(format!("ERR: {}", line));
            }
        });

        let mut final_stdout = String::new();
        let mut final_stderr = String::new();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if let Ok(msg) = rx.try_recv() {
                if msg.starts_with("OUT: ") {
                    final_stdout.push_str(&msg[5..]);
                    final_stdout.push('\n');
                } else {
                    final_stderr.push_str(&msg[5..]);
                    final_stderr.push('\n');
                }
            }
            
            if let Ok(Some(status)) = child.try_wait() {
                // Drain remaining output
                while let Ok(msg) = rx.try_recv() {
                    if msg.starts_with("OUT: ") {
                        final_stdout.push_str(&msg[5..]);
                        final_stdout.push('\n');
                    } else {
                        final_stderr.push_str(&msg[5..]);
                        final_stderr.push('\n');
                    }
                }
                return Ok(ExecutionResult {
                    stdout: final_stdout,
                    stderr: final_stderr,
                    exit_code: status.code().unwrap_or(0),
                    duration: start.elapsed(),
                });
            }
            
            thread::sleep(Duration::from_millis(10));
        }

        // Timeout hit
        let _ = child.kill();
        Err(format!("Command timed out after {} seconds", timeout_secs))
    }
}
