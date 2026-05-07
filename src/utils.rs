//! DevUtils Extra Utilities

use std::fs;
use std::path::PathBuf;

pub fn ensure_dir(dir: &str) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    Ok(())
}

pub fn clamp_number(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn clamp_int(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn normalize_path(p: &str) -> String {
    if p.starts_with('/') || p.contains(':') {
        p.to_string()
    } else {
        format!("./{}", p)
    }
}

pub fn expand_path(p: &str) -> PathBuf {
    if p.starts_with('~') {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        PathBuf::from(home).join(&p[1..])
    } else {
        PathBuf::from(p)
    }
}

pub fn file_age_ms(path: &str) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(modified).ok()?;
    Some(duration.as_millis() as u64)
}

pub fn file_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_json<T: serde::Serialize>(path: &str, value: &T) -> std::io::Result<()> {
    let content = serde_json::to_string_pretty(value).ok().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "JSON serialization failed")
    })?;
    fs::write(path, content)?;
    Ok(())
}

pub fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins % 60)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

pub fn get_system_info() -> String {
    let mut info = String::new();

    info.push_str(&format!("OS: {}\n", std::env::consts::OS));
    info.push_str(&format!("Arch: {}\n", std::env::consts::ARCH));

    info.push_str(&format!("Shell: powershell\n"));

    info
}
