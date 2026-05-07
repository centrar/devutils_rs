//! Interactive Terminal Debugger
//!
//! Provides debugging capabilities in terminal without GUI
//! Uses a combination of:
//! - Log-based debugging with trace levels
//! - Breakpoint management via code instrumentation
//! - Test-driven debugging (write failing test, run, fix)
//! - Stack trace analysis
//! - Variable inspection via logging

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Debug event types
#[derive(Debug, Clone)]
pub enum DebugEvent {
    BreakpointHit { file: String, line: u32, id: u64 },
    Log { level: LogLevel, message: String, file: String, line: u32 },
    Error { message: String, stack: Vec<String> },
    Watch { name: String, old_value: String, new_value: String },
}

/// Log levels for debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Debug,
        }
    }
    
    pub fn to_str(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Breakpoint configuration
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: u64,
    pub file: String,
    pub line: u32,
    pub condition: Option<String>,
    pub enabled: bool,
    pub hit_count: u64,
}

/// Watch point for variable tracking
#[derive(Debug, Clone)]
pub struct WatchPoint {
    pub id: u64,
    pub expression: String,
    pub old_value: Option<String>,
    pub enabled: bool,
}

/// Debug session state
pub struct DebugSession {
    pub breakpoints: Vec<Breakpoint>,
    pub watchpoints: Vec<WatchPoint>,
    pub log_level: LogLevel,
    pub logs: Vec<DebugEvent>,
    pub running: bool,
    pub current_file: Option<String>,
}

impl Default for DebugSession {
    fn default() -> Self {
        Self {
            breakpoints: Vec::new(),
            watchpoints: Vec::new(),
            log_level: LogLevel::Debug,
            logs: Vec::new(),
            running: false,
            current_file: None,
        }
    }
}

/// Debugger state
static mut DEBUG_SESSION: Option<DebugSession> = None;

/// Initialize debug session
pub fn init_session() {
    unsafe {
        DEBUG_SESSION = Some(DebugSession::default());
    }
}

/// Get debug session (unsafe due to static mut - acceptable for debug module)
#[allow(static_mut_refs)]
fn get_session() -> Option<&'static mut DebugSession> {
    unsafe { DEBUG_SESSION.as_mut() }
}

/// Add a breakpoint
pub fn add_breakpoint(file: &str, line: u32, condition: Option<&str>) -> u64 {
    if let Some(session) = get_session() {
        let id = session.breakpoints.len() as u64 + 1;
        session.breakpoints.push(Breakpoint {
            id,
            file: file.to_string(),
            line,
            condition: condition.map(|s| s.to_string()),
            enabled: true,
            hit_count: 0,
        });
        id
    } else {
        0
    }
}

/// Remove a breakpoint
pub fn remove_breakpoint(id: u64) -> bool {
    if let Some(session) = get_session() {
        if let Some(pos) = session.breakpoints.iter().position(|b| b.id == id) {
            session.breakpoints.remove(pos);
            return true;
        }
    }
    false
}

/// Enable/disable breakpoint
pub fn set_breakpoint_enabled(id: u64, enabled: bool) -> bool {
    if let Some(session) = get_session() {
        if let Some(bp) = session.breakpoints.iter_mut().find(|b| b.id == id) {
            bp.enabled = enabled;
            return true;
        }
    }
    false
}

/// Add a watch point
pub fn add_watch(expression: &str) -> u64 {
    if let Some(session) = get_session() {
        let id = session.watchpoints.len() as u64 + 1;
        session.watchpoints.push(WatchPoint {
            id,
            expression: expression.to_string(),
            old_value: None,
            enabled: true,
        });
        id
    } else {
        0
    }
}

/// Log a debug message
pub fn log(level: LogLevel, message: &str, file: &str, line: u32) {
    if let Some(session) = get_session() {
        let level_idx = match level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        };
        let session_level_idx = match session.log_level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        };
        
        if level_idx >= session_level_idx {
            session.logs.push(DebugEvent::Log {
                level,
                message: message.to_string(),
                file: file.to_string(),
                line,
            });
        }
    }
}

/// Macro for easy logging
#[macro_export]
macro_rules! debug_log {
    ($level:expr, $msg:expr) => {
        devutils_debug::log($level, $msg, file!(), line!());
    };
    ($level:expr, $fmt:expr, $($arg:tt)*) => {
        devutils_debug::log($level, &format!($fmt, $($arg)*), file!(), line!());
    };
}

/// Set log level
pub fn set_log_level(level: LogLevel) {
    if let Some(session) = get_session() {
        session.log_level = level;
    }
}

/// Clear all logs
pub fn clear_logs() {
    if let Some(session) = get_session() {
        session.logs.clear();
    }
}

/// Get all logs
pub fn get_logs() -> Vec<DebugEvent> {
    get_session().map(|s| s.logs.clone()).unwrap_or_default()
}

/// Print logs to terminal
pub fn print_logs() {
    if let Some(session) = get_session() {
        for event in &session.logs {
            match event {
                DebugEvent::Log { level, message, file, line } => {
                    println!("[{}] {}:{} - {}", level.to_str(), file, line, message);
                }
                DebugEvent::Error { message, stack } => {
                    println!("[ERROR] {}\n  Stack: {}", message, stack.join("\n  "));
                }
                _ => {}
            }
        }
    }
}

/// Generate a debug build of a Rust project
pub fn debug_build(project_path: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["build", "--debug"])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run cargo debug build: {}", e))?;
    
    if output.status.success() {
        Ok("Debug build successful".to_string())
    } else {
        Err(format!(
            "Debug build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Run tests with debug output
pub fn debug_test(project_path: &Path, test_name: Option<&str>) -> Result<String, String> {
    let mut args = vec!["test".to_string(), "--".to_string(), "--nocapture".to_string()];
    if let Some(name) = test_name {
        args.push(name.to_string());
    }
    
    let output = Command::new("cargo")
        .args(&args)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run cargo test: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    if output.status.success() {
        Ok(format!("Tests passed:\n{}\n{}", stdout, stderr))
    } else {
        Err(format!("Tests failed:\n{}\n{}", stdout, stderr))
    }
}

/// Run debugger command (requires lldb or gdb)
pub fn run_external_debugger(
    project_path: &Path,
    binary_path: &Path,
    break_line: Option<u32>,
) -> Result<String, String> {
    // Try lldb first, then gdb
    let debugger = if Command::new("lldb").arg("--version").output().is_ok() {
        "lldb"
    } else if Command::new("gdb").arg("--version").output().is_ok() {
        "gdb"
    } else {
        return Err("No debugger found (install lldb or gdb)".to_string());
    };
    
    let mut cmd = Command::new(debugger);
    
    match debugger {
        "lldb" => {
            let mut lldb_cmd = format!(
                "breakpoint set --file {} --line {}\nrun\nbt\n",
                binary_path.display(),
                break_line.unwrap_or(1)
            );
            if let Some(line) = break_line {
                lldb_cmd = format!(
                    "breakpoint set --file {} --line {}\nrun\nbt\n",
                    binary_path.display(),
                    line
                );
            } else {
                lldb_cmd = format!("run\nbt\n");
            }
            cmd.arg("--batch").arg("-o").arg(&lldb_cmd).arg(binary_path);
        }
        "gdb" => {
            if let Some(line) = break_line {
                cmd.arg("-ex").arg(format!("break {}:{}", binary_path.display(), line));
            }
            cmd.arg("-ex").arg("run");
            cmd.arg("-ex").arg("bt");
            cmd.arg(binary_path);
        }
        _ => unreachable!(),
    };
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to run debugger: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Attach to a running process
pub fn attach_to_process(pid: u32) -> Result<String, String> {
    let debugger = if Command::new("lldb").arg("--version").output().is_ok() {
        "lldb"
    } else if Command::new("gdb").arg("--version").output().is_ok() {
        "gdb"
    } else {
        return Err("No debugger found".to_string());
    };
    
    let output = match debugger {
        "lldb" => {
            Command::new("lldb")
                .args(["-p", &pid.to_string(), "-o", "bt"])
                .output()
        }
        "gdb" => {
            Command::new("gdb")
                .args(["-p", &pid.to_string(), "-ex", "bt", "-ex", "quit"])
                .output()
        }
        _ => return Err("No debugger found".to_string()),
    }.map_err(|e| format!("Failed to attach: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Generate backtrace for current thread
pub fn backtrace() -> Vec<String> {
    // For a simple implementation, we return the recent log entries
    get_logs()
        .iter()
        .filter_map(|e| match e {
            DebugEvent::Log { file, line, message, .. } => {
                Some(format!("  at {}:{} - {}", file, line, message))
            }
            _ => None,
        })
        .collect()
}

/// Print session status
pub fn status() -> String {
    if let Some(session) = get_session() {
        let mut output = String::from("=== Debug Session Status ===\n");
        output.push_str(&format!("Running: {}\n", session.running));
        output.push_str(&format!("Log Level: {}\n", session.log_level.to_str()));
        output.push_str(&format!("Breakpoints: {}\n", session.breakpoints.len()));
        output.push_str(&format!("Watchpoints: {}\n", session.watchpoints.len()));
        output.push_str(&format!("Log entries: {}\n", session.logs.len()));
        
        if !session.breakpoints.is_empty() {
            output.push_str("\nBreakpoints:\n");
            for bp in &session.breakpoints {
                output.push_str(&format!(
                    "  #{} {}:{} (hit: {}, {})\n",
                    bp.id,
                    bp.file,
                    bp.line,
                    bp.hit_count,
                    if bp.enabled { "enabled" } else { "disabled" }
                ));
            }
        }
        
        if !session.watchpoints.is_empty() {
            output.push_str("\nWatchpoints:\n");
            for wp in &session.watchpoints {
                output.push_str(&format!(
                    "  #{} '{}' (current: {:?})\n",
                    wp.id, wp.expression, wp.old_value
                ));
            }
        }
        
        output
    } else {
        "No debug session active. Call init_session() first.".to_string()
    }
}

/// Instrument code with debug logging (helper for generating instrumented code)
pub fn instrument_code(file_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Simple instrumentation - add debug logs to function entries
    let mut result = String::new();
    
    for line in content.lines() {
        result.push_str(line);
        result.push('\n');
        
        // Add entry logging to functions
        if line.starts_with("fn ") || line.starts_with("pub fn ") {
            let func_name = line.split('(').next().unwrap_or("unknown");
            result.push_str(&format!(
                "    debug_log!(LogLevel::Debug, \"Entering {}\");\n",
                func_name.trim()
            ));
        }
    }
    
    Ok(result)
}

/// Run debugger REPL
pub fn start_repl() {
    use std::io::{self, BufRead};
    
    init_session();
    
    println!("=== DevUtils Debug REPL ===");
    println!("Commands: break <file:line>, watch <expr>, run, continue, step, print, status, quit");
    
    let stdin = io::stdin();
    let mut input = String::new();
    
    loop {
        print!("debug> ");
        io::stdout().flush().unwrap();
        
        input.clear();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0] {
            "break" | "b" => {
                if let Some(loc) = parts.get(1) {
                    if let Some((file, line)) = loc.split_once(':') {
                        let line_num: u32 = line.parse().unwrap_or(1);
                        let id = add_breakpoint(file, line_num, None);
                        println!("Breakpoint #{} set at {}:{}", id, file, line_num);
                    }
                }
            }
            "watch" | "w" => {
                if let Some(expr) = parts.get(1) {
                    let id = add_watch(expr);
                    println!("Watch #{} set for '{}'", id, expr);
                }
            }
            "run" => {
                println!("Starting debug session...");
            }
            "continue" | "c" => {
                println!("Continuing execution...");
                break;
            }
            "step" | "s" => {
                println!("Stepping...");
            }
            "print" | "p" => {
                if let Some(expr) = parts.get(1) {
                    println!("{} = (value unavailable without running program)", expr);
                }
            }
            "status" => {
                println!("{}", status());
            }
            "logs" | "l" => {
                print_logs();
            }
            "quit" | "q" => {
                println!("Exiting debug REPL");
                break;
            }
            "help" | "h" => {
                println!("Commands:");
                println!("  break <file:line> - Set breakpoint");
                println!("  watch <expr> - Set watchpoint");
                println!("  run - Start debug session");
                println!("  continue - Continue execution");
                println!("  step - Step to next line");
                println!("  print <expr> - Print variable");
                println!("  status - Show debug session status");
                println!("  logs - Show log entries");
                println!("  quit - Exit REPL");
            }
            _ => {
                println!("Unknown command: {}. Type 'help' for commands.", parts[0]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_session() {
        init_session();
        let session = get_session();
        assert!(session.is_some());
    }

    #[test]
    fn test_add_breakpoint() {
        init_session();
        let id = add_breakpoint("test.rs", 10, None);
        assert!(id > 0);
    }

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::to_str(&LogLevel::Info), "INFO");
    }
}