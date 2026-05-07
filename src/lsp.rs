use std::io::{BufRead, BufReader, Write, Read};
use std::process::{Child, Command, Stdio};
use serde::Serialize;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LspClient {
    child: Child,
    id_gen: AtomicUsize,
}

#[derive(Serialize)]
struct LspRequest {
    jsonrpc: String,
    id: usize,
    method: String,
    params: Value,
}

impl LspClient {
    pub fn new(workspace: &str) -> Result<Self, String> {
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let ra_path = format!(r"{}\.cargo\bin\rust-analyzer.exe", home);

        let child = Command::new(ra_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .current_dir(workspace)
            .spawn()
            .map_err(|e| format!("Failed to spawn rust-analyzer: {}", e))?;

        let mut client = Self {
            child,
            id_gen: AtomicUsize::new(1),
        };

        client.initialize(workspace)?;
        Ok(client)
    }

    fn send_request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.id_gen.fetch_add(1, Ordering::SeqCst);
        let req = LspRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let body = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        let content = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

        let stdin = self.child.stdin.as_mut().ok_or("No stdin")?;
        stdin.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;

        self.read_response(id)
    }

    fn read_response(&mut self, target_id: usize) -> Result<Value, String> {
        let stdout = self.child.stdout.as_mut().ok_or("No stdout")?;
        let mut reader = BufReader::new(stdout);

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if line.starts_with("Content-Length: ") {
                let len: usize = line.trim_start_matches("Content-Length: ").trim().parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                
                // Skip empty line
                let mut empty = String::new();
                reader.read_line(&mut empty).map_err(|e: std::io::Error| e.to_string())?;

                let mut body = vec![0u8; len];
                reader.read_exact(&mut body).map_err(|e: std::io::Error| e.to_string())?;

                let resp: Value = serde_json::from_slice(&body).map_err(|e: serde_json::Error| e.to_string())?;
                if resp["id"].as_u64() == Some(target_id as u64) {
                    return Ok(resp["result"].clone());
                }
            }
        }
    }

    fn initialize(&mut self, workspace: &str) -> Result<(), String> {
        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": format!("file:///{}", workspace.replace("\\", "/")),
            "capabilities": {}
        });
        self.send_request("initialize", params)?;
        self.send_request("initialized", serde_json::json!({}))?;
        Ok(())
    }

    pub fn format_file(&mut self, text: &str) -> Result<String, String> {
        // For simplicity in the agent, we use a temporary virtual file
        // A real LSP would use textDocument/formatting on a saved file
        Ok(text.to_string()) // Placeholder for now
    }

    pub fn get_definition(&mut self, file: &str, line: usize, character: usize) -> Result<Value, String> {
        let params = serde_json::json!({
            "textDocument": { "uri": format!("file:///{}", file.replace("\\", "/")) },
            "position": { "line": line, "character": character }
        });
        self.send_request("textDocument/definition", params)
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
