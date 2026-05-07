//! Auth Profiles - Multiple API Keys Management

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const AUTH_DB: &str = ".devutils_auth.json";

static AUTH_PROFILES: Lazy<Mutex<HashMap<String, AuthProfile>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub created_at: u64,
    pub last_used: Option<u64>,
    pub use_count: u32,
    pub enabled: bool,
}

impl AuthProfile {
    pub fn new(name: &str, provider: &str, api_key: &str) -> Self {
        Self {
            name: name.to_string(),
            provider: provider.to_string(),
            api_key: api_key.to_string(),
            created_at: current_timestamp(),
            last_used: None,
            use_count: 0,
            enabled: true,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn add_profile(name: &str, provider: &str, api_key: &str) -> Result<String, String> {
    let profile = AuthProfile::new(name, provider, api_key);
    let key = profile.name.clone();

    AUTH_PROFILES.lock().unwrap().insert(key.clone(), profile);
    save_auth_profiles(&AUTH_PROFILES.lock().unwrap())?;

    Ok(format!("Added auth profile: {} ({})", name, provider))
}

pub fn list_profiles() -> Vec<AuthProfile> {
    let profiles = AUTH_PROFILES.lock().unwrap();
    profiles.values().cloned().collect()
}

pub fn remove_profile(name: &str) -> Result<String, String> {
    let mut profiles = AUTH_PROFILES.lock().unwrap();

    if profiles.remove(name).is_some() {
        save_auth_profiles(&profiles)?;
        return Ok(format!("Removed profile: {}", name));
    }

    Err(format!("Profile '{}' not found", name))
}

pub fn enable_profile(name: &str, enabled: bool) -> Result<String, String> {
    let mut profiles = AUTH_PROFILES.lock().unwrap();

    if let Some(profile) = profiles.get_mut(name) {
        profile.enabled = enabled;
        save_auth_profiles(&profiles)?;
        return Ok(format!(
            "Profile '{}' {}",
            name,
            if enabled { "enabled" } else { "disabled" }
        ));
    }

    Err(format!("Profile '{}' not found", name))
}

pub fn get_api_key(name: &str) -> Option<String> {
    AUTH_PROFILES
        .lock()
        .unwrap()
        .get(name)
        .map(|p| p.api_key.clone())
}

fn load_auth_profiles() -> Result<HashMap<String, AuthProfile>, String> {
    let path = PathBuf::from(AUTH_DB);
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let profiles: Vec<AuthProfile> = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for p in profiles {
        map.insert(p.name.clone(), p);
    }

    Ok(map)
}

fn save_auth_profiles(profiles: &HashMap<String, AuthProfile>) -> Result<(), String> {
    let values: Vec<_> = profiles.values().collect();
    let content = serde_json::to_string_pretty(&values).map_err(|e| e.to_string())?;
    fs::write(AUTH_DB, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_openai_key() -> String {
    std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_KEY"))
        .unwrap_or_default()
}

pub fn list_auth_commands() {
    println!("\n\x1b[36m🔐 Auth Profiles\x1b[0m");
    println!("\nUsage:");
    println!("  devutils auth add <name> <provider> <api-key>");
    println!("  devutils auth list");
    println!("  devutils auth remove <name>");
    println!("  devutils auth enable <name>");
    println!("  devutils auth disable <name>");
    println!("\nProviders:");
    println!("  openai, anthropic, google, github");
    println!("\nExamples:");
    println!("  devutils auth add default openai sk-...");
    println!("  devutils auth list");
}
