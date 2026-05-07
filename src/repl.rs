use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Simple interactive REPL for DevUtils
pub struct Repl {
    history: Vec<String>,
    prompt: String,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            prompt: "devutils> ".to_string(),
        }
    }

    pub fn run(&mut self) -> i32 {
        println!("DevUtils Interactive REPL");
        println!("Type 'help' for available commands, 'exit' to quit");
        println!();

        loop {
            // Print prompt
            print!("{}", self.prompt);
            io::stdout().flush().unwrap();

            // Read input
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => {
                    // EOF (Ctrl+D)
                    println!("\nGoodbye!");
                    break;
                }
                Err(_) => {
                    eprintln!("Error reading input");
                    continue;
                }
                Ok(_) => {
                    // Process input
                    let input = input.trim_end();
                    
                    // Handle multi-line commands (ending with \)
                    let mut full_input = input.to_string();
                    while full_input.ends_with('\\') {
                        full_input.pop(); // Remove the backslash
                        full_input.pop(); // Remove newline if present
                        print!("... ");
                        io::stdout().flush().unwrap();
                        
                        let mut more = String::new();
                        if io::stdin().read_line(&mut more).is_err() || more.is_empty() {
                            break;
                        }
                        full_input.push_str(&more.trim_end());
                    }

                    // Skip empty input
                    if full_input.trim().is_empty() {
                        continue;
                    }

                    // Add to history
                    self.history.push(full_input.clone());

                    // Handle special commands
                    match full_input.trim() {
                        "help" | "?" => self.print_help(),
                        "exit" | "quit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "clear" => {
                            // Clear screen (ANSI escape)
                            print!("\x1B[2J\x1B[1;1H");
                        }
                        "history" => self.print_history(),
                        "" => continue,
                        cmd => {
                            // Execute as devutils command
                            if let Err(e) = self.execute_command(cmd) {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
            }
        }

        0
    }

    fn print_help(&self) {
        println!("\nAvailable commands:");
        println!("  help, ?          - Show this help");
        println!("  exit, quit       - Exit REPL");
        println!("  clear            - Clear screen");
        println!("  history          - Show command history");
        println!("  Any devutils command (e.g., status, grep \"pattern\", generate \"hello world\")");
        println!();
    }

    fn print_history(&self) {
        if self.history.is_empty() {
            println!("No history");
            return;
        }
        println!("\nCommand history:");
        for (i, cmd) in self.history.iter().enumerate() {
            println!("  {:3}: {}", i + 1, cmd);
        }
        println!();
    }

    fn execute_command(&self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Split into command and args
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        // Find the devutils executable
        let exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("devutils"));

        // Execute devutils with the command
        let output = std::process::Command::new(&exe)
            .args(&parts)
            .output()?;

        // Print output
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}