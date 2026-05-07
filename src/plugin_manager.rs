//! Real Plugin System with Marketplace
//! 
//! Features:
//! - Install/uninstall plugins from marketplace
//! - Local plugin management
//! - Plugin execution with arguments
//! - Plugin configuration

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: String,
    pub commands: Vec<PluginCommand>,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub script: String,
    pub args: Vec<String>,
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    registry_url: String,
}

impl PluginManager {
    pub fn new() -> Self {
        let plugins_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("devutils")
            .join("plugins");
        
        fs::create_dir_all(&plugins_dir).ok();
        
        Self {
            plugins_dir,
            registry_url: "https://plugins.devutils.ai".to_string(),
        }
    }

    pub fn install(&self, name: &str) -> Result<String, String> {
        // Fetch plugin from registry
        let plugin = self.fetch_plugin(name)?;
        
        // Create plugin directory
        let plugin_dir = self.plugins_dir.join(&plugin.name);
        fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to create plugin dir: {}", e))?;
        
        // Save plugin manifest
        let manifest_path = plugin_dir.join("manifest.json");
        let manifest = serde_json::json!({
            "name": plugin.name,
            "version": plugin.version,
            "description": plugin.description,
            "author": plugin.author,
            "homepage": plugin.homepage,
            "commands": plugin.commands.iter().map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "description": c.description,
                    "script": c.script,
                    "args": c.args
                })
            }).collect::<Vec<_>>()
        });
        
        fs::write(&manifest_path, manifest.to_string())
            .map_err(|e| format!("Failed to write manifest: {}", e))?;
        
        // Save plugin script
        if !plugin.commands.is_empty() {
            let script_path = plugin_dir.join("main.sh");
            fs::write(&script_path, &plugin.commands[0].script)
                .map_err(|e| format!("Failed to write script: {}", e))?;
        }
        
        Ok(format!("Installed plugin '{}' v{}", plugin.name, plugin.version))
    }

    pub fn uninstall(&self, name: &str) -> Result<String, String> {
        let plugin_dir = self.plugins_dir.join(name);
        
        if !plugin_dir.exists() {
            return Err(format!("Plugin '{}' is not installed", name));
        }
        
        fs::remove_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to remove plugin: {}", e))?;
        
        Ok(format!("Uninstalled plugin '{}'", name))
    }

    pub fn list_installed(&self) -> Vec<Plugin> {
        let mut plugins = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(plugin) = self.load_plugin(&manifest_path) {
                        plugins.push(plugin);
                    }
                }
            }
        }
        
        plugins
    }

    pub fn search(&self, query: &str) -> Result<Vec<Plugin>, String> {
        // In a real implementation, this would query the marketplace API
        // For now, return mock results
        let mock_plugins = vec![
            Plugin {
                name: "git-clean".to_string(),
                version: "1.0.0".to_string(),
                description: "Clean git branches".to_string(),
                author: "DevUtils".to_string(),
                homepage: "https://github.com/devutils/git-clean".to_string(),
                commands: vec![],
                installed: false,
            },
            Plugin {
                name: "docker-helper".to_string(),
                version: "2.1.0".to_string(),
                description: "Docker utilities".to_string(),
                author: "DevUtils".to_string(),
                homepage: "https://github.com/devutils/docker-helper".to_string(),
                commands: vec![],
                installed: false,
            },
            Plugin {
                name: "api-test".to_string(),
                version: "1.5.0".to_string(),
                description: "API testing tool".to_string(),
                author: "DevUtils".to_string(),
                homepage: "https://github.com/devutils/api-test".to_string(),
                commands: vec![],
                installed: false,
            },
        ];
        
        let query_lower = query.to_lowercase();
        Ok(mock_plugins.into_iter()
            .filter(|p| p.name.to_lowercase().contains(&query_lower) 
                || p.description.to_lowercase().contains(&query_lower))
            .collect())
    }

    pub fn execute(&self, name: &str, _command: &str, args: &[String]) -> Result<String, String> {
        let plugin_dir = self.plugins_dir.join(name);
        
        if !plugin_dir.exists() {
            return Err(format!("Plugin '{}' is not installed", name));
        }
        
        let script_path = plugin_dir.join("main.sh");
        if !script_path.exists() {
            return Err(format!("Plugin '{}' has no executable script", name));
        }
        
        // Execute the plugin script
        let output = Command::new("bash")
            .arg(&script_path)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute plugin: {}", e))?;
        
        let result = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        
        Ok(result)
    }

    fn fetch_plugin(&self, name: &str) -> Result<Plugin, String> {
        // In a real implementation, this would fetch from the registry API
        // For now, return mock plugin data
        Ok(Plugin {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Plugin {}", name),
            author: "DevUtils".to_string(),
            homepage: format!("https://github.com/devutils/{}", name),
            commands: vec![PluginCommand {
                name: "default".to_string(),
                description: "Default command".to_string(),
                script: format!("echo 'Running {}'", name),
                args: vec![],
            }],
            installed: false,
        })
    }

    fn load_plugin(&self, manifest_path: &Path) -> Result<Plugin, String> {
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;
        
        Ok(Plugin {
            name: json["name"].as_str().unwrap_or("unknown").to_string(),
            version: json["version"].as_str().unwrap_or("0.0.0").to_string(),
            description: json["description"].as_str().unwrap_or("").to_string(),
            author: json["author"].as_str().unwrap_or("unknown").to_string(),
            homepage: json["homepage"].as_str().unwrap_or("").to_string(),
            commands: vec![],
            installed: true,
        })
    }
}

// CLI commands
pub fn plugin_install(name: &str) -> String {
    let manager = PluginManager::new();
    match manager.install(name) {
        Ok(msg) => msg,
        Err(e) => format!("Error: {}", e),
    }
}

pub fn plugin_uninstall(name: &str) -> String {
    let manager = PluginManager::new();
    match manager.uninstall(name) {
        Ok(msg) => msg,
        Err(e) => format!("Error: {}", e),
    }
}

pub fn plugin_list() -> String {
    let manager = PluginManager::new();
    let plugins = manager.list_installed();
    
    if plugins.is_empty() {
        return "No plugins installed".to_string();
    }
    
    let mut output = String::from("\n🔌 Installed Plugins:\n\n");
    for plugin in plugins {
        output.push_str(&format!("  {} v{} - {}\n", plugin.name, plugin.version, plugin.description));
    }
    output
}

pub fn plugin_search(query: &str) -> String {
    let manager = PluginManager::new();
    match manager.search(query) {
        Ok(plugins) => {
            if plugins.is_empty() {
                return format!("No plugins found for '{}'", query);
            }
            
            let mut output = format!("\n🔍 Search results for '{}':\n\n", query);
            for plugin in plugins {
                output.push_str(&format!(
                    "  {} v{} - {}\n    by {} | {}\n",
                    plugin.name,
                    plugin.version,
                    plugin.description,
                    plugin.author,
                    plugin.homepage
                ));
            }
            output
        }
        Err(e) => format!("Error: {}", e),
    }
}

pub fn plugin_execute(name: &str, command: &str, args: &[String]) -> String {
    let manager = PluginManager::new();
    match manager.execute(name, command, args) {
        Ok(result) => result,
        Err(e) => format!("Error: {}", e),
    }
}