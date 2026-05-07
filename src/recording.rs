//! Recording/Sharing - Terminal session recording

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub content: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Input,
    Output,
    Command,
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub duration_secs: u64,
    pub events: Vec<TerminalEvent>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub os: String,
    pub shell: String,
    pub devutils_version: String,
    pub exit_code: Option<i32>,
}

pub struct SessionRecorder {
    events: VecDeque<TerminalEvent>,
    id: String,
    start_time: u64,
    recording: bool,
}

impl SessionRecorder {
    pub fn new() -> Self {
        let id = Self::generate_id();
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            events: VecDeque::new(),
            id,
            start_time,
            recording: false,
        }
    }

    fn generate_id() -> String {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("rec_{}", &format!("{:x}", now)[..8])
    }

    pub fn start(&mut self) {
        self.events.clear();
        self.recording = true;
        self.add_event(EventType::Info, "Recording started".to_string());
    }

    pub fn stop(&mut self) -> Recording {
        self.recording = false;
        self.add_event(EventType::Info, "Recording stopped".to_string());

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.start_time;

        Recording {
            id: self.id.clone(),
            title: "Terminal Session".to_string(),
            created_at: self.start_time,
            duration_secs: duration,
            events: self.events.iter().cloned().collect(),
            metadata: Self::get_metadata(),
        }
    }

    fn get_metadata() -> SessionMetadata {
        SessionMetadata {
            os: std::env::consts::OS.to_string(),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "powershell".to_string()),
            devutils_version: "1.0.0".to_string(),
            exit_code: None,
        }
    }

    pub fn add_event(&mut self, event_type: EventType, content: String) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.events.push_back(TerminalEvent {
            timestamp,
            event_type,
            content,
            duration_ms: None,
        });
    }

    pub fn record_input(&mut self, cmd: &str) {
        if self.recording {
            self.add_event(EventType::Input, cmd.to_string());
        }
    }

    pub fn record_output(&mut self, output: &str) {
        if self.recording {
            self.add_event(EventType::Output, output.to_string());
        }
    }

    pub fn save(&self, path: Option<&str>) -> Result<PathBuf, String> {
        let path = match path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from(format!("session_{}.json", &self.id[..6])),
        };

        let recording = Recording {
            id: self.id.clone(),
            title: "Terminal Session".to_string(),
            created_at: self.start_time,
            duration_secs: 0,
            events: self.events.iter().cloned().collect(),
            metadata: Self::get_metadata(),
        };

        let json = serde_json::to_string_pretty(&recording).map_err(|e| e.to_string())?;

        fs::write(&path, json).map_err(|e| e.to_string())?;

        Ok(path)
    }

    pub fn export_html(&self) -> String {
        let mut html = String::new();

        html.push_str(
            r#"<!DOCTYPE html>
<html><head><meta charset="utf-8">
<title>DevUtils Session</title>
<style>
body { background: #1e1e1e; color: #d4d4d4; font-family: monospace; padding: 20px; }
.input { color: #4ec9b0; }
.output { color: #d4d4d4; }
.command { color: #569cd6; font-weight: bold; }
.error { color: #f44747; }
.info { color: #6a9955; font-style: italic; }
pre { margin: 0; white-space: pre-wrap; word-wrap: break-word; }
</style></head><body>
"#,
        );

        for event in &self.events {
            let class = match event.event_type {
                EventType::Input => "input",
                EventType::Output => "output",
                EventType::Command => "command",
                EventType::Error => "error",
                EventType::Info => "info",
            };
            html.push_str(&format!(
                "<pre class=\"{}\">{}</pre>\n",
                class, event.content
            ));
        }

        html.push_str("</body></html>");
        html
    }

    pub fn export_gif(&self) -> Result<String, String> {
        Ok("GIF export requires external tool. Run: devutils share --gif".to_string())
    }
}

pub fn share_session(file: &str) -> Result<String, String> {
    let content = fs::read_to_string(file).map_err(|e| format!("Failed to read: {}", e))?;

    let recording: Recording =
        serde_json::from_str(&content).map_err(|e| format!("Invalid session file: {}", e))?;

    let html = SessionRecorder::new().export_html();

    let output_path = PathBuf::from(format!("{}.html", recording.id));
    fs::write(&output_path, &html).map_err(|e| format!("Failed to write: {}", e))?;

    Ok(format!(
        "Session exported to: {}\n\nHTML file can be shared or hosted.",
        output_path.display()
    ))
}

pub fn view_session(file: &str) -> Result<String, String> {
    let content = fs::read_to_string(file).map_err(|e| format!("Failed to read: {}", e))?;

    let recording: Recording =
        serde_json::from_str(&content).map_err(|e| format!("Invalid session file: {}", e))?;

    let mut output = String::new();
    output.push_str(&format!(
        "\n\x1b[36m📺 Session: {}\x1b[0m\n",
        recording.title
    ));
    output.push_str(&format!("Duration: {}s\n", recording.duration_secs));
    output.push_str(&format!("Events: {}\n\n", recording.events.len()));

    for event in &recording.events {
        let prefix = match event.event_type {
            EventType::Input => "\x1b[32m$\x1b[0m",
            EventType::Output => "  ",
            EventType::Command => "\x1b[33m>\x1b[0m",
            EventType::Error => "\x1b[31mERR\x1b[0m",
            EventType::Info => "\x1b[90mINFO\x1b[0m",
        };
        output.push_str(&format!("{} {}\n", prefix, event.content));
    }

    Ok(output)
}
