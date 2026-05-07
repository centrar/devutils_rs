//! Plugin Ecosystem Loader - Native Plugin System
//! Loads plugins from GitHub, marketplace, or local directory
//! Makes external CLI tools native to DevUtils

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub repository: String,
    pub commands: Vec<PluginCommand>,
    pub dependencies: Vec<String>,
    pub entry_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub handler: String,
    pub args: Vec<PluginArg>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    registry: HashMap<String, LoadedPlugin>,
}

impl PluginManager {
    pub fn new() -> Result<Self, String> {
        let plugins_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("devutils")
            .join("plugins");
        
        fs::create_dir_all(&plugins_dir).ok();
        
        Ok(Self {
            plugins_dir,
            registry: HashMap::new(),
        })
    }

    /// Load all plugins from plugins directory
    pub fn load_all(&mut self) -> Result<(), String> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.plugins_dir)
            .map_err(|e| format!("Failed to read plugins dir: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    self.load_plugin(&manifest_path)?;
                }
            }
        }
        
        Ok(())
    }

    /// Load a single plugin
    fn load_plugin(&mut self, manifest_path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;
        
        let plugin = LoadedPlugin {
            manifest,
            path: manifest_path.parent().unwrap().to_path_buf(),
            enabled: true,
        };
        
        self.registry.insert(plugin.manifest.name.clone(), plugin);
        Ok(())
    }

    /// Install plugin from GitHub
    pub fn install_from_github(&mut self, repo: &str) -> Result<String, String> {
        // Format: owner/repo or just repo (assumes devutils org)
        let repo_url = if repo.contains('/') {
            format!("https://github.com/{}.git", repo)
        } else {
            format!("https://github.com/devutils/plugin-{}.git", repo)
        };
        
        let plugin_name = repo.split('/').last().unwrap_or(repo);
        let plugin_path = self.plugins_dir.join(plugin_name);
        
        // Clone repository
        let output = Command::new("git")
            .args(&["clone", &repo_url, plugin_path.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to clone: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Clone failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Load the plugin
        let manifest_path = plugin_path.join("plugin.json");
        if manifest_path.exists() {
            self.load_plugin(&manifest_path)?;
            Ok(format!("Installed plugin '{}' from {}", plugin_name, repo_url))
        } else {
            Err("No plugin.json found in repository".to_string())
        }
    }

    /// Execute a plugin command
    pub fn execute(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<String, String> {
        let plugin = self.registry.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;
        
        let cmd = plugin.manifest.commands.iter()
            .find(|c| c.name == command)
            .ok_or_else(|| format!("Command '{}' not found in plugin '{}'", command, plugin_name))?;
        
        // Execute the command handler
        let mut cmd_parts: Vec<&str> = cmd.handler.split_whitespace().collect();
        cmd_parts.extend(args.iter().map(|s| s.as_str()));
        
        let output = Command::new(cmd_parts[0])
            .args(&cmd_parts[1..])
            .output()
            .map_err(|e| format!("Failed to execute: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// List all available plugins
    pub fn list(&self) -> Vec<&PluginManifest> {
        self.registry.values().map(|p| &p.manifest).collect()
    }

    /// Search plugins
    pub fn search(&self, query: &str) -> Vec<&PluginManifest> {
        self.list().into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query.to_lowercase()) ||
                p.description.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }
}

// CLI commands
pub fn plugin_install(repo: &str) -> String {
    let mut manager = match PluginManager::new() {
        Ok(m) => m,
        Err(e) => return format!("Error: {}", e),
    };
    
    match manager.install_from_github(repo) {
        Ok(msg) => msg,
        Err(e) => format!("Error: {}", e),
    }
}

pub fn plugin_list() -> String {
    let manager = match PluginManager::new() {
        Ok(m) => m,
        Err(e) => return format!("Error: {}", e),
    };
    
    let plugins = manager.list();
    if plugins.is_empty() {
        return "No plugins installed".to_string();
    }
    
    let mut output = String::from("\n🔌 Installed Plugins:\n\n");
    for plugin in plugins {
        output.push_str(&format!(
            "  {} v{} - {}\n    by {} | {}\n",
            plugin.name, plugin.version, plugin.description, plugin.author, plugin.repository
        ));
    }
    output
}

pub fn plugin_search(query: &str) -> String {
    let manager = match PluginManager::new() {
        Ok(m) => m,
        Err(e) => return format!("Error: {}", e),
    };
    
    let results = manager.search(query);
    if results.is_empty() {
        return format!("No plugins found for '{}'", query);
    }
    
    let mut output = format!("\n🔍 Search results for '{}':\n\n", query);
    for plugin in results {
        output.push_str(&format!(
            "  {} v{} - {}\n    by {} | {}\n",
            plugin.name, plugin.version, plugin.description, plugin.author, plugin.repository
        ));
    }
    output
}

pub fn plugin_execute(plugin: &str, command: &str, args: &[String]) -> String {
    let manager = match PluginManager::new() {
        Ok(m) => m,
        Err(e) => return format!("Error: {}", e),
    };
    
    match manager.execute(plugin, command, args) {
        Ok(result) => result,
        Err(e) => format!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }
}