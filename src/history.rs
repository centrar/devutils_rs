use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

static HISTORY: Mutex<Option<History>> = Mutex::new(None);

pub struct History {
    path: PathBuf,
    commands: Vec<HistoryEntry>,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

impl History {
    pub fn new(path: PathBuf) -> Self {
        let commands = Self::load_from_file(&path);
        Self { path, commands }
    }

    fn load_from_file(path: &PathBuf) -> Vec<HistoryEntry> {
        let mut entries = Vec::new();
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Some(entry) = HistoryEntry::from_line(&line) {
                    entries.push(entry);
                }
            }
        }
        entries
    }

    pub fn add(&mut self, command: String, exit_code: i32, duration_ms: u64) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.commands.push(HistoryEntry {
            timestamp,
            command,
            exit_code,
            duration_ms,
        });
    }

    pub fn save(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| format!("Failed to open history file: {}", e))?;

        for entry in &self.commands {
            writeln!(file, "{}", entry.to_line())
                .map_err(|e| format!("Failed to write: {}", e))?;
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.commands
            .iter()
            .filter(|e| e.command.contains(query))
            .collect()
    }

    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        self.commands.iter().rev().take(n).collect()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

impl HistoryEntry {
    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            Some(HistoryEntry {
                timestamp: parts[0].parse().ok()?,
                command: parts[1].to_string(),
                exit_code: parts[2].parse().ok()?,
                duration_ms: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn to_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.timestamp, self.command, self.exit_code, self.duration_ms
        )
    }
}

pub fn init_history() -> History {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let history_dir = data_dir.join("devutils");
    fs::create_dir_all(&history_dir).ok();
    let history_path = history_dir.join("history");

    History::new(history_path)
}

pub fn add_history(command: String, exit_code: i32, duration_ms: u64) {
    if let Ok(mut guard) = HISTORY.lock() {
        if guard.is_none() {
            *guard = Some(init_history());
        }
        if let Some(ref mut history) = *guard {
            history.add(command, exit_code, duration_ms);
            let _ = history.save();
        }
    }
}

pub fn get_history() -> Vec<HistoryEntry> {
    if let Ok(guard) = HISTORY.lock() {
        if let Some(ref history) = *guard {
            return history.commands.clone();
        }
    }
    Vec::new()
}

pub fn search_history(query: &str) -> Vec<HistoryEntry> {
    if let Ok(guard) = HISTORY.lock() {
        if let Some(ref history) = *guard {
            return history.search(query).into_iter().cloned().collect();
        }
    }
    Vec::new()
}

pub fn clear_history() {
    if let Ok(mut guard) = HISTORY.lock() {
        if let Some(ref mut history) = *guard {
            history.clear();
            let _ = history.save();
        }
    }
}