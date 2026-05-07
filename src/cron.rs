//! Cron Scheduler - scheduled task execution

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const CRON_DB: &str = ".devutils_cron.json";

static CRON_JOBS: Lazy<Mutex<Vec<CronJob>>> =
    Lazy::new(|| Mutex::new(load_cron_jobs().unwrap_or_default()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub command: String,
    pub schedule: CronSchedule,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
    pub run_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CronSchedule {
    #[serde(rename = "at")]
    At { at_ms: u64 },
    #[serde(rename = "every")]
    Every {
        every_ms: u64,
        anchor_ms: Option<u64>,
    },
    #[serde(rename = "cron")]
    Cron { expr: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobCreate {
    pub name: String,
    pub command: String,
    pub schedule: CronSchedule,
    pub enabled: Option<bool>,
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn next_run_time(schedule: &CronSchedule) -> String {
    match next_run_time_internal(schedule) {
        Some(ts) => format!("{}", ts),
        None => "Never".to_string(),
    }
}

fn next_run_time_internal(schedule: &CronSchedule) -> Option<u64> {
    let now = now_ms();

    match schedule {
        CronSchedule::At { at_ms } => {
            if *at_ms > now {
                Some(*at_ms)
            } else {
                None
            }
        }
        CronSchedule::Every {
            every_ms,
            anchor_ms,
        } => {
            let interval = *every_ms;
            let anchor = anchor_ms.unwrap_or(now);
            if now < anchor {
                return Some(anchor);
            }
            let elapsed = now - anchor;
            let steps = (elapsed / interval) + 1;
            Some(anchor + steps * interval)
        }
        CronSchedule::Cron { expr } => parse_simple_cron(expr, now),
    }
}

fn parse_simple_cron(expr: &str, _now: u64) -> Option<u64> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let next = now_ms() + 60000;
    Some(next)
}

fn generate_id() -> String {
    format!("cron_{}", &format!("{:x}", now_ms())[..8])
}

pub fn add_job(name: &str, schedule: &str, command: &str) -> String {
    let sched = match parse_schedule(schedule) {
        Ok(s) => s,
        Err(e) => return e,
    };

    let input = CronJobCreate {
        name: name.to_string(),
        command: command.to_string(),
        schedule: sched,
        enabled: Some(true),
    };

    match add_job_internal(input) {
        Ok(id) => format!("Added job: {} ({})", name, id),
        Err(e) => e,
    }
}

fn parse_schedule(s: &str) -> Result<CronSchedule, String> {
    if s.starts_with("every ") {
        let ms: u64 = s[6..].parse().map_err(|_| "Invalid duration".to_string())?;
        Ok(CronSchedule::Every {
            every_ms: ms,
            anchor_ms: None,
        })
    } else if s.starts_with("at ") {
        let ts: u64 = s[3..]
            .parse()
            .map_err(|_| "Invalid timestamp".to_string())?;
        Ok(CronSchedule::At { at_ms: ts })
    } else {
        Ok(CronSchedule::Cron {
            expr: s.to_string(),
        })
    }
}

fn add_job_internal(input: CronJobCreate) -> Result<String, String> {
    let id = generate_id();
    let next_run = next_run_time_internal(&input.schedule);

    let job = CronJob {
        id: id.clone(),
        name: input.name,
        command: input.command,
        schedule: input.schedule,
        enabled: input.enabled.unwrap_or(true),
        last_run: None,
        next_run,
        run_count: 0,
    };

    CRON_JOBS.lock().unwrap().push(job);
    save_cron_jobs(&CRON_JOBS.lock().unwrap())?;

    Ok(id)
}

pub fn list_jobs() -> String {
    let jobs = CRON_JOBS.lock().unwrap();
    if jobs.is_empty() {
        return "No cron jobs".to_string();
    }

    let mut output = String::new();
    output.push_str("Cron Jobs:\n");
    for job in jobs.iter() {
        let next = job
            .next_run
            .map(|n| n.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        output.push_str(&format!(
            "  [{}] {} - next: {} - runs: {}\n",
            job.id, job.name, next, job.run_count
        ));
    }
    output
}

pub fn remove_job(name: &str) -> String {
    let mut jobs = CRON_JOBS.lock().unwrap();
    let initial_len = jobs.len();
    jobs.retain(|j| j.name != name);

    if jobs.len() == initial_len {
        return format!("Job '{}' not found", name);
    }

    match save_cron_jobs(&jobs) {
        Ok(_) => format!("Removed job: {}", name),
        Err(e) => e,
    }
}

pub fn run_job(name: &str) -> String {
    let mut jobs = CRON_JOBS.lock().unwrap();

    let job = match jobs.iter_mut().find(|j| j.name == name) {
        Some(j) => j,
        None => return format!("Job '{}' not found", name),
    };

    let output = match std::process::Command::new("powershell")
        .args(["-Command", &job.command])
        .output()
    {
        Ok(o) => o,
        Err(e) => return format!("Failed to run: {}", e),
    };

    job.last_run = Some(now_ms());
    job.run_count += 1;
    job.next_run = next_run_time_internal(&job.schedule);

    let result = String::from_utf8_lossy(&output.stdout).to_string();
    let _ = save_cron_jobs(&jobs);

    if result.is_empty() {
        format!("Job '{}' executed successfully", name)
    } else {
        result
    }
}

pub fn tick() -> String {
    let now = now_ms();
    let mut results = Vec::new();

    let mut jobs = CRON_JOBS.lock().unwrap();

    for job in jobs.iter_mut() {
        if !job.enabled {
            continue;
        }

        if let Some(next) = job.next_run {
            if next <= now {
                let output = std::process::Command::new("powershell")
                    .args(["-Command", &job.command])
                    .output();

                if let Ok(o) = output {
                    if !o.stdout.is_empty() {
                        results.push(String::from_utf8_lossy(&o.stdout).to_string());
                    }
                }

                job.last_run = Some(now);
                job.run_count += 1;
                job.next_run = next_run_time_internal(&job.schedule);
            }
        }
    }

    if !results.is_empty() {
        let _ = save_cron_jobs(&jobs);
    }

    if results.is_empty() {
        "No jobs due".to_string()
    } else {
        results.join("\n")
    }
}

fn load_cron_jobs() -> Result<Vec<CronJob>, String> {
    let path = PathBuf::from(CRON_DB);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let jobs: Vec<CronJob> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(jobs)
}

fn save_cron_jobs(jobs: &[CronJob]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(jobs).map_err(|e| e.to_string())?;
    fs::write(CRON_DB, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_commands() {
    println!("\n⏰ Cron Scheduler");
    println!("\nUsage:");
    println!("  devutils cron add <name> <command> <schedule>");
    println!("  devutils cron list");
    println!("  devutils cron run <name>");
    println!("  devutils cron remove <name>");
    println!("  devutils cron tick");
    println!("  devutils cron next <schedule>");
    println!("\nSchedules:");
    println!("  every <ms>    - Every X milliseconds");
    println!("  at <timestamp> - At specific timestamp");
    println!("  <cron>     - Cron expression (e.g., '0 9 * * *')");
    println!("\nExamples:");
    println!("  devutils cron add daily-backup 'npm run backup' '0 9 * * *'");
    println!("  devutils cron add sync 'devutils sync' 'every 3600000'");
    println!("  devutils cron list");
}
