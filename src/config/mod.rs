//! Configuration Management Module
//! Persistent configuration stored in ~/.devutils/config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub plugins: PluginConfig,
    pub enterprise: EnterpriseConfig,
    pub ui: UiConfig,
    pub editor: String,
    pub shell: String,
    pub keybindings: String,
    pub pager: String,
    pub color_theme: String,
    pub max_history: u32,
    pub auto_update: bool,
    pub sync_provider: String,
    pub sync_path: Option<String>,
    pub default_ai_provider: String,
    pub default_ai_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub provider: String,
    pub api_keys: HashMap<String, String>,
    pub default_model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub registry_url: String,
    pub installed: Vec<String>,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    pub enabled: bool,
    pub sso_url: Option<String>,
    pub audit_enabled: bool,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub verbose: bool,
    pub color: bool,
    pub progress_bars: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        let mut api_keys = HashMap::new();
        // Load from environment if available
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            api_keys.insert("openai".to_string(), key);
        }
        if let Ok(key) = std::env::var("NVIDIA_API_KEY") {
            api_keys.insert("nvidia".to_string(), key);
        }
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            api_keys.insert("deepseek".to_string(), key);
        }
        
        Self {
            provider: "openai".to_string(),
            api_keys,
            default_model: "gpt-4".to_string(),
            timeout_secs: 60,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://github.com/devutils".to_string(),
            installed: Vec::new(),
            auto_update: false,
        }
    }
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sso_url: None,
            audit_enabled: true,
            team_id: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            color: true,
            progress_bars: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            plugins: PluginConfig::default(),
            enterprise: EnterpriseConfig::default(),
            ui: UiConfig::default(),
            editor: "code".to_string(),
            shell: "bash".to_string(),
            keybindings: "vim".to_string(),
            pager: "less".to_string(),
            color_theme: "auto".to_string(),
            max_history: 1000,
            auto_update: true,
            sync_provider: "none".to_string(),
            sync_path: None,
            default_ai_provider: "openai".to_string(),
            default_ai_model: "gpt-4".to_string(),
        }
    }
}

impl Config {
    /// Get config directory path
    pub fn config_dir() -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "Could not determine home directory")?;
        
        let config_dir = PathBuf::from(home).join(".devutils");
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        Ok(config_dir)
    }
    
    /// Get config file path
    pub fn config_path() -> Result<PathBuf, String> {
        Ok(Self::config_dir()?.join("config.toml"))
    }
    
    /// Load configuration from file
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;
        
        if !path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&path, &content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        Ok(())
    }
    
    /// Get API key for provider
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        self.api.api_keys.get(provider).cloned()
            .or_else(|| std::env::var(&format!("{}_API_KEY", provider.to_uppercase())).ok())
    }
    
    /// Set API key for provider
    pub fn set_api_key(&mut self, provider: &str, key: &str) {
        self.api.api_keys.insert(provider.to_string(), key.to_string());
    }
    
    /// Get current provider
    pub fn get_provider(&self) -> String {
        self.api.provider.clone()
    }
    
    /// Set provider
    pub fn set_provider(&mut self, provider: &str) {
        self.api.provider = provider.to_string();
    }
}

/// CLI command for config management
pub fn run_config_cmd(subcmd: &str, args: &[String]) -> Result<String, String> {
    match subcmd {
        "show" => {
            let config = Config::load()?;
            Ok(format!("Current Configuration:\n{}", toml::to_string_pretty(&config).unwrap()))
        }
        "set" => {
            if args.len() < 2 {
                return Err("Usage: devutils config set <key> <value>".to_string());
            }
            
            let mut config = Config::load()?;
            let key = &args[0];
            let value = &args[1];
            
            // Simple key setting (e.g., "api.provider" = "openai")
            if key == "api.provider" {
                config.set_provider(value);
            } else if key.starts_with("api.api_keys.") {
                let provider = key.trim_start_matches("api.api_keys.").to_string();
                config.set_api_key(&provider, value);
            } else {
                return Err(format!("Unknown config key: {}", key));
            }
            
            config.save()?;
            Ok(format!("✅ Set {} = {}", key, value))
        }
        "get" => {
            if args.is_empty() {
                return Err("Usage: devutils config get <key>".to_string());
            }
            
            let config = Config::load()?;
            let key = &args[0];
            
            if key == "api.provider" {
                Ok(config.get_provider())
            } else if let Some(provider) = key.strip_prefix("api.api_keys.") {
                config.get_api_key(provider)
                    .ok_or_else(|| format!("No API key set for {}", provider))
            } else {
                Err(format!("Unknown config key: {}", key))
            }
        }
        "unset" => {
            if args.is_empty() {
                return Err("Usage: devutils config unset <key>".to_string());
            }
            
            let mut config = Config::load()?;
            let key = &args[0];
            
            if key.starts_with("api.api_keys.") {
                let provider = key.trim_start_matches("api.api_keys.").to_string();
                config.api.api_keys.remove(&provider);
            } else {
                return Err(format!("Cannot unset key: {}", key));
            }
            
            config.save()?;
            Ok(format!("✅ Unset {}", key))
        }
        "list" => {
            let config = Config::load()?;
            Ok(format!("API Provider: {}\nAPI Keys: {:?}\nDefault Model: {}", 
                config.api.provider, 
                config.api.api_keys.keys().collect::<Vec<_>>(),
                config.api.default_model
            ))
        }
        _ => Err(format!("Unknown config command: {}", subcmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.api.provider, "openai");
    }
}

/// Get config file path
pub fn get_config_path() -> Result<PathBuf, String> {
    Config::config_path()
}

/// Print config to console
pub fn print_config(cfg: &Config) {
    println!(" DevUtils Configuration");
    println!();
    println!(" default_ai_provider   {}", cfg.default_ai_provider);
    println!(" default_ai_model      {}", cfg.default_ai_model);
    println!(" editor                {}", cfg.editor);
    println!(" shell                 {}", cfg.shell);
    println!(" keybindings           {}", cfg.keybindings);
    println!(" pager                 {}", cfg.pager);
    println!(" color_theme           {}", cfg.color_theme);
    println!(" max_history           {}", cfg.max_history);
    println!(" auto_update           {}", cfg.auto_update);
    println!(" sync_provider         {}", cfg.sync_provider);
    println!(" sync_path             {:?}", cfg.sync_path);
    println!();
    println!(" Config file: {:?}", Config::config_path().unwrap_or_else(|_| PathBuf::from(".devutils.toml")));
}

/// Set a config value
pub fn set_value(cfg: &mut Config, key: &str, value: &str) -> Result<(), String> {
    match key {
        "editor" => cfg.editor = value.to_string(),
        "shell" => cfg.shell = value.to_string(),
        "keybindings" => cfg.keybindings = value.to_string(),
        "pager" => cfg.pager = value.to_string(),
        "color_theme" => cfg.color_theme = value.to_string(),
        "max_history" => cfg.max_history = value.parse().map_err(|_| "Invalid number")?,
        "auto_update" => cfg.auto_update = value.parse().map_err(|_| "Invalid boolean")?,
        "sync_provider" => cfg.sync_provider = value.to_string(),
        "sync_path" => cfg.sync_path = Some(value.to_string()),
        "default_ai_provider" => cfg.default_ai_provider = value.to_string(),
        "default_ai_model" => cfg.default_ai_model = value.to_string(),
        "api.provider" => cfg.api.provider = value.to_string(),
        "api.default_model" => cfg.api.default_model = value.to_string(),
        _ => return Err(format!("Unknown config key: {}", key)),
    }
    Ok(())
}
