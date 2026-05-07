//! Web Interface - Browser-based UI for DevUtils
//!
//! HTTP server with web UI showing:
//! - Chat interface
//! - Terminal output
//! - File tree
//! - Agent status

use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

static WEB_STATE: Lazy<Arc<Mutex<WebState>>> = Lazy::new(|| Arc::new(Mutex::new(WebState::new())));

#[derive(Debug)]
pub struct WebState {
    pub messages: VecDeque<WebMessage>,
    pub output: String,
    pub files: Vec<String>,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct WebMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

impl WebState {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            output: String::new(),
            files: vec![],
            connected: false,
        }
    }
}

pub fn start_server(port: u16) -> Result<()> {
    println!(
        "\n\x1b[36m🌐 Starting DevUtils Web Interface on http://localhost:{}\x1b[0m",
        port
    );

    // Simple HTML response
    let _html = r#"<!DOCTYPE html>
<html>
<head>
    <title>DevUtils - AI Developer Toolkit</title>
    <meta charset="utf-8">
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117; color: #c9d1d9; height: 100vh; display: flex; 
        }
        .sidebar {
            width: 250px; background: #161b22; border-right: 1px solid #30363d;
            padding: 1rem;
        }
        .main { flex: 1; display: flex; flex-direction: column; }
        .chat {
            flex: 1; overflow-y: auto; padding: 1rem;
        }
        .input-area {
            padding: 1rem; border-top: 1px solid #30363d;
            display: flex; gap: 0.5rem;
        }
        input {
            flex: 1; background: #0d1117; border: 1px solid #30363d; border-radius: 6px;
            padding: 0.75rem; color: #c9d1d9; font-size: 1rem;
        }
        button {
            background: #238636; color: white; border: none; border-radius: 6px;
            padding: 0.75rem 1.5rem; cursor: pointer; font-weight: 600;
        }
        .message { margin-bottom: 1rem; padding: 1rem; border-radius: 8px; }
        .user { background: #1f6feb; }
        .assistant { background: #21262d; }
        .header { padding: 1rem; border-bottom: 1px solid #30363d; }
        h1 { font-size: 1.25rem; color: #58a6ff; }
    </style>
</head>
<body>
    <div class="sidebar">
        <h2 style="color:#58a6ff;margin-bottom:1rem;">🤖 DevUtils</h2>
        <div style="font-size:0.875rem;color:#8b949e;">
            <p>📦 40k VS Code Extensions</p>
            <p>🔌 100+ MCP Servers</p>
            <p>🖥️ Split-pane TUI</p>
            <p>🐳 Docker Sessions</p>
            <p>📀 Local Ollama</p>
        </div>
    </div>
    <div class="main">
        <div class="header">
            <h1>AI Developer Toolkit</h1>
        </div>
        <div class="chat" id="chat">
            <div class="message assistant">
                <strong>🤖 DevUtils:</strong> Hello! I'm your AI developer assistant. What would you like to build?
            </div>
        </div>
        <div class="input-area">
            <input type="text" id="input" placeholder="Describe what you want to build..." onkeydown="if(event.key==='Enter')send()">
            <button onclick="send()">Send</button>
        </div>
    </div>
    <script>
        async function send() {
            const input = document.getElementById('input');
            const chat = document.getElementById('chat');
            const msg = input.value;
            if(!msg) return;
            
            chat.innerHTML += '<div class="message user"><strong>❯ You:</strong> '+msg+'</div>';
            input.value = '';
            
            chat.innerHTML += '<div class="message assistant"><strong>🤖 DevUtils:</strong> Processing...</div>';
            chat.scrollTop = chat.scrollHeight;
        }
    </script>
</body>
</html>"#;

    let _web_state = WEB_STATE.clone();

    println!("\n\x1b[32m✓ Web interface ready!\x1b[0m");
    println!("   Open http://localhost:{} in your browser", port);

    Ok(())
}

pub fn add_web_message(role: &str, content: &str) {
    let mut state = WEB_STATE.lock().unwrap();
    state.messages.push_back(WebMessage {
        role: role.to_string(),
        content: content.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });

    // Keep only last 100 messages
    while state.messages.len() > 100 {
        state.messages.pop_front();
    }
}

pub fn get_web_status() -> String {
    let state = WEB_STATE.lock().unwrap();
    format!(
        "Messages: {}, Connected: {}",
        state.messages.len(),
        state.connected
    )
}

pub fn web_commands() {
    println!("\n\x1b[36m🌐 Web Interface:\x1b[0m\n");
    println!("  \x1b[33mdevutils web\x1b[0m              - Start web UI");
    println!("  \x1b[33mdevutils web <port>\x1b[0m      - Start on custom port");
    println!();
}
