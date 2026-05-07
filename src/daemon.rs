use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Daemon {
    port: u16,
    state: Arc<Mutex<DaemonState>>,
}

#[derive(Default)]
pub struct DaemonState {
    pub requests: usize,
    pub last_request: Option<String>,
}

impl Daemon {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(Mutex::new(DaemonState::default())),
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        println!("DevUtils Daemon running on {}", addr);
        println!("Connect with: devutils --connect localhost:{}", self.port);
        println!("Press Ctrl+C to stop");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let state = Arc::clone(&self.state);
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, &state) {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn stats(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "DevUtils Daemon Stats:\n  Requests: {}\n  Last request: {}",
            state.requests,
            state.last_request.as_deref().unwrap_or("none")
        )
    }
}

fn handle_connection(mut stream: TcpStream, state: &Arc<Mutex<DaemonState>>) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    // Read request
    let mut buffer = vec![0u8; 4096];
    let n = stream
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;

    let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    // Update stats
    {
        let mut state = state.lock().unwrap();
        state.requests += 1;
        state.last_request = Some(request.clone());
    }

    // Process request
    let response = if request.is_empty() {
        "OK\n".to_string()
    } else {
        // Parse command and execute
        let parts: Vec<&str> = request.split_whitespace().collect();
        if parts.is_empty() {
            "OK\n".to_string()
        } else {
            // Execute devutils command
            let exe = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("devutils"));

            let output = std::process::Command::new(&exe)
                .args(&parts[1..])
                .output()
                .map_err(|e| format!("Failed to execute: {}", e))?;

            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }
    };

    // Send response
    stream
        .write_all(response.as_bytes())
        .map_err(|e| format!("Failed to write: {}", e))?;

    Ok(())
}

pub fn start_daemon(port: u16) -> Result<(), String> {
    let daemon = Daemon::new(port);
    daemon.run()
}

pub fn daemon_status() -> String {
    // Try to connect to daemon and get status
    let addr = format!("127.0.0.1:{}", 7890);
    match TcpStream::connect(&addr) {
        Ok(_stream) => format!("Daemon running on {}", addr),
        Err(_) => "Daemon not running".to_string(),
    }
}

pub fn stop_daemon() -> Result<(), String> {
    // Connect and send stop signal
    let addr = "127.0.0.1:7890";
    let mut stream =
        TcpStream::connect(addr).map_err(|e| format!("Failed to connect: {}", e))?;
    stream
        .write_all(b"STOP\n")
        .map_err(|e| format!("Failed to send stop: {}", e))?;
    Ok(())
}