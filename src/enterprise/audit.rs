//! Audit Logs Module

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    Login,
    Logout,
    FileCreate,
    FileModify,
    FileDelete,
    CodeGenerate,
    CodeExecute,
    AgentRun,
    PluginInstall,
    PluginExecute,
    ConfigChange,
    UserCreate,
    UserDelete,
    PermissionChange,
}

impl ToString for AuditEventType {
    fn to_string(&self) -> String {
        match self {
            AuditEventType::Login => "login".to_string(),
            AuditEventType::Logout => "logout".to_string(),
            AuditEventType::FileCreate => "file.create".to_string(),
            AuditEventType::FileModify => "file.modify".to_string(),
            AuditEventType::FileDelete => "file.delete".to_string(),
            AuditEventType::CodeGenerate => "code.generate".to_string(),
            AuditEventType::CodeExecute => "code.execute".to_string(),
            AuditEventType::AgentRun => "agent.run".to_string(),
            AuditEventType::PluginInstall => "plugin.install".to_string(),
            AuditEventType::PluginExecute => "plugin.execute".to_string(),
            AuditEventType::ConfigChange => "config.change".to_string(),
            AuditEventType::UserCreate => "user.create".to_string(),
            AuditEventType::UserDelete => "user.delete".to_string(),
            AuditEventType::PermissionChange => "permission.change".to_string(),
        }
    }
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub details: String,
    pub ip_address: Option<String>,
    pub success: bool,
    pub session_id: Option<String>,
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        user_id: &str,
        action: &str,
        resource: &str,
        details: &str,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            event_type: event_type.to_string(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            details: details.to_string(),
            ip_address: None,
            success: true,
            session_id: None,
        }
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn failed(mut self) -> Self {
        self.success = false;
        self
    }
}

/// Audit logger
pub struct AuditLogger {
    log_path: PathBuf,
    max_size_mb: u64,
    retention_days: u32,
}

impl AuditLogger {
    pub fn new() -> Self {
        let log_path = std::env::var("DEVUTILS_AUDIT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut path = std::env::current_dir().unwrap_or_default();
                path.push(".devutils_audit.log");
                path
            });

        Self {
            log_path,
            max_size_mb: 100,
            retention_days: 90,
        }
    }

    /// Log an audit event
    pub fn log(&self, event: &AuditEvent) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| format!("Failed to open audit log: {}", e))?;

        let json = serde_json::to_string(event)
            .map_err(|e| format!("Failed to serialize audit event: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write audit event: {}", e))?;

        // Rotate log if needed
        self.rotate_if_needed()?;

        Ok(())
    }

    /// Query audit logs
    pub fn query(
        &self,
        user_id: Option<&str>,
        event_type: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<Vec<AuditEvent>, String> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_path)
            .map_err(|e| format!("Failed to open audit log: {}", e))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read audit log: {}", e))?;

            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                // Filter by user
                if let Some(uid) = user_id {
                    if event.user_id != uid {
                        continue;
                    }
                }

                // Filter by event type
                if let Some(et) = event_type {
                    if event.event_type != et {
                        continue;
                    }
                }

                // Filter by time range
                if let Some(start) = start_time {
                    if event.timestamp < start {
                        continue;
                    }
                }

                if let Some(end) = end_time {
                    if event.timestamp > end {
                        continue;
                    }
                }

                events.push(event);
            }
        }

        // Sort by timestamp (newest first)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(events)
    }

    /// Get audit statistics
    pub fn stats(&self) -> Result<AuditStats, String> {
        let events = self.query(None, None, None, None)?;

        let mut stats = AuditStats::default();
        stats.total_events = events.len();

        for event in &events {
            if event.success {
                stats.successful += 1;
            } else {
                stats.failed += 1;
            }
        }

        Ok(stats)
    }

    /// Rotate log if needed
    fn rotate_if_needed(&self) -> Result<(), String> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(&self.log_path)
            .map_err(|e| format!("Failed to get audit log metadata: {}", e))?;

        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb > self.max_size_mb {
            // Rotate log
            let rotated = format!("{}.1", self.log_path.display());
            std::fs::rename(&self.log_path, &rotated)
                .map_err(|e| format!("Failed to rotate audit log: {}", e))?;
        }

        Ok(())
    }

    /// Clean old logs
    pub fn cleanup_old_logs(&self) -> Result<u32, String> {
        // Implementation for cleaning old logs
        Ok(0)
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit statistics
#[derive(Debug, Default)]
pub struct AuditStats {
    pub total_events: usize,
    pub successful: usize,
    pub failed: usize,
}

/// Global audit logger instance using OnceLock for thread-safety
static AUDIT_LOGGER: OnceLock<AuditLogger> = OnceLock::new();

/// Get global audit logger
pub fn get_audit_logger() -> &'static AuditLogger {
    AUDIT_LOGGER.get_or_init(|| AuditLogger::new())
}

/// Log an audit event (convenience function)
pub fn log_audit(
    event_type: AuditEventType,
    user_id: &str,
    action: &str,
    resource: &str,
    details: &str,
) {
    let event = AuditEvent::new(event_type, user_id, action, resource, details);
    if let Some(logger) = AUDIT_LOGGER.get() {
        let _ = logger.log(&event);
    }
}

/// CLI command for audit logs
pub fn run_audit_cmd(subcmd: &str, args: &[String]) -> Result<String, String> {
    let logger = AuditLogger::new();

    match subcmd {
        "list" | "ls" => {
            let events = logger.query(None, None, None, None)?;
            let mut output = String::new();
            output.push_str(&format!("Audit Logs ({} events)\n", events.len()));
            output.push_str(&"=".repeat(50));
            output.push('\n');

            for event in events.iter().take(20) {
                output.push_str(&format!(
                    "[{}] {} - {} - {}\n",
                    event.timestamp, event.event_type, event.user_id, event.action
                ));
            }

            if events.len() > 20 {
                output.push_str(&format!("\n... and {} more", events.len() - 20));
            }

            Ok(output)
        }
        "user" => {
            let user_id = args.first().ok_or("Please specify user ID")?;
            let events = logger.query(Some(user_id), None, None, None)?;
            Ok(format!("Found {} events for user {}", events.len(), user_id))
        }
        "stats" => {
            let stats = logger.stats()?;
            Ok(format!(
                "Audit Statistics\n\
                 ================\n\
                 Total Events: {}\n\
                 Successful: {}\n\
                 Failed: {}",
                stats.total_events, stats.successful, stats.failed
            ))
        }
        "export" => {
            let output_file = args.first().ok_or("Please specify output file")?;
            let events = logger.query(None, None, None, None)?;

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output_file)
                .map_err(|e| format!("Failed to open output file: {}", e))?;

            let event_count = events.len();
            for event in events {
                let json = serde_json::to_string(&event)
                    .map_err(|e| format!("Failed to serialize event: {}", e))?;
                writeln!(file, "{}", json)
                    .map_err(|e| format!("Failed to write event: {}", e))?;
            }

            Ok(format!("Exported {} events to {}", event_count, output_file))
        }
        _ => Err(format!("Unknown audit command: {}", subcmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::Login,
            "user123",
            "login",
            "system",
            "User logged in",
        );

        assert_eq!(event.user_id, "user123");
        assert_eq!(event.event_type, "login");
        assert!(event.success);
    }

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new();
        assert!(true);
    }
}
