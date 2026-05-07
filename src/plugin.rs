//! Plugin System - Extensible DevUtils plugins

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    config_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub commands: Vec<PluginCommand>,
    pub hooks: Vec<Hook>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<CommandArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub event: String,
    pub action: String,
}

impl PluginManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("devutils");

        let mut manager = Self {
            plugins: HashMap::new(),
            config_dir: config_dir.clone(),
        };

        manager.load_plugins();
        manager
    }

    fn load_plugins(&mut self) {
        let plugin_dirs = vec![self.config_dir.join("plugins"), PathBuf::from("./plugins")];

        for dir in plugin_dirs {
            if dir.exists() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(name) = path.file_name() {
                                let plugin_name = name.to_string_lossy().to_string();
                                if let Ok(mut plugin) = load_plugin(&path) {
                                    plugin.enabled = true;
                                    self.plugins.insert(plugin_name, plugin);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.register_builtin_plugins();
    }

    fn register_builtin_plugins(&mut self) {
        self.plugins.insert("git".to_string(), GIT_PLUGIN.clone());
        self.plugins
            .insert("docker".to_string(), DOCKER_PLUGIN.clone());
        self.plugins.insert("k8s".to_string(), K8S_PLUGIN.clone());
        self.plugins.insert("ai".to_string(), AI_PLUGIN.clone());
    }

    pub fn list(&self) -> Vec<(&str, &Plugin)> {
        self.plugins
            .iter()
            .filter(|(_, p)| p.enabled)
            .map(|(k, p)| (k.as_str(), p))
            .collect()
    }

    pub fn enable(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = true;
            save_plugin_config(&self.config_dir, name, plugin)?;
        }
        Ok(())
    }

    pub fn disable(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = false;
            save_plugin_config(&self.config_dir, name, plugin)?;
        }
        Ok(())
    }

    pub fn execute(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<String> {
        let plugin = self
            .plugins
            .get(plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_name))?;

        for cmd in &plugin.commands {
            if cmd.name == command {
                return Ok(format!(
                    "Executing {} {} with {:?}",
                    plugin_name, command, args
                ));
            }
        }

        Ok(format!(
            "Command '{}' not found in plugin '{}'",
            command, plugin_name
        ))
    }

    pub fn get_commands(&self, plugin_name: &str) -> Vec<&PluginCommand> {
        self.plugins
            .get(plugin_name)
            .map(|p| p.commands.iter().collect())
            .unwrap_or_default()
    }

    pub fn install(&mut self, plugin: Plugin) -> Result<()> {
        let dir = self.config_dir.join("plugins").join(&plugin.name);
        fs::create_dir_all(&dir)?;

        let manifest = dir.join("plugin.toml");
        let content = toml::to_string(&plugin)?;
        fs::write(manifest, content)?;

        self.plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    pub fn uninstall(&mut self, name: &str) -> Result<()> {
        if let Some(_plugin) = self.plugins.remove(name) {
            let dir = self.config_dir.join("plugins").join(name);
            if dir.exists() {
                fs::remove_dir_all(dir)?;
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

fn load_plugin(path: &PathBuf) -> Result<Plugin> {
    let manifest = path.join("plugin.toml");
    if !manifest.exists() {
        return Ok(Plugin {
            name: "unknown".to_string(),
            version: "0.0.0".to_string(),
            description: "Unknown plugin".to_string(),
            author: "unknown".to_string(),
            commands: vec![],
            hooks: vec![],
            enabled: false,
        });
    }

    let content = fs::read_to_string(manifest)?;
    let plugin: Plugin = toml::from_str(&content).unwrap_or(Plugin {
        name: "unknown".to_string(),
        version: "0.0.0".to_string(),
        description: "Unknown plugin".to_string(),
        author: "unknown".to_string(),
        commands: vec![],
        hooks: vec![],
        enabled: false,
    });
    Ok(plugin)
}

fn save_plugin_config(config_dir: &PathBuf, name: &str, plugin: &Plugin) -> Result<()> {
    let dir = config_dir.join("plugins").join(name);
    fs::create_dir_all(&dir)?;

    let manifest = dir.join("plugin.toml");
    let content = toml::to_string_pretty(plugin)?;
    fs::write(manifest, content)?;
    Ok(())
}

lazy_static::lazy_static! {
    static ref GIT_PLUGIN: Plugin = Plugin {
        name: "git".to_string(),
        version: "1.0.0".to_string(),
        description: "Git integration plugin".to_string(),
        author: "DevUtils Team".to_string(),
        commands: vec![
            PluginCommand {
                name: "status".to_string(),
                description: "Show git status".to_string(),
                args: vec![],
            },
            PluginCommand {
                name: "commit".to_string(),
                description: "Create a commit".to_string(),
                args: vec![
                    CommandArg {
                        name: "message".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "push".to_string(),
                description: "Push to remote".to_string(),
                args: vec![],
            },
            PluginCommand {
                name: "pull".to_string(),
                description: "Pull from remote".to_string(),
                args: vec![],
            },
            PluginCommand {
                name: "branch".to_string(),
                description: "List/create branches".to_string(),
                args: vec![
                    CommandArg {
                        name: "name".to_string(),
                        arg_type: "string".to_string(),
                        required: false,
                        default: None,
                    },
                ],
            },
        ],
        hooks: vec![
            Hook {
                event: "pre-commit".to_string(),
                action: "lint".to_string(),
            },
            Hook {
                event: "post-commit".to_string(),
                action: "notify".to_string(),
            },
        ],
        enabled: true,
    };

    static ref DOCKER_PLUGIN: Plugin = Plugin {
        name: "docker".to_string(),
        version: "1.0.0".to_string(),
        description: "Docker management plugin".to_string(),
        author: "DevUtils Team".to_string(),
        commands: vec![
            PluginCommand {
                name: "ps".to_string(),
                description: "List containers".to_string(),
                args: vec![],
            },
            PluginCommand {
                name: "run".to_string(),
                description: "Run a container".to_string(),
                args: vec![
                    CommandArg {
                        name: "image".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "build".to_string(),
                description: "Build image".to_string(),
                args: vec![
                    CommandArg {
                        name: "tag".to_string(),
                        arg_type: "string".to_string(),
                        required: false,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "logs".to_string(),
                description: "Container logs".to_string(),
                args: vec![
                    CommandArg {
                        name: "container".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "exec".to_string(),
                description: "Execute in container".to_string(),
                args: vec![
                    CommandArg {
                        name: "container".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                    CommandArg {
                        name: "command".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
        ],
        hooks: vec![],
        enabled: true,
    };

    static ref K8S_PLUGIN: Plugin = Plugin {
        name: "k8s".to_string(),
        version: "1.0.0".to_string(),
        description: "Kubernetes management plugin".to_string(),
        author: "DevUtils Team".to_string(),
        commands: vec![
            PluginCommand {
                name: "pods".to_string(),
                description: "List pods".to_string(),
                args: vec![],
            },
            PluginCommand {
                name: "deploy".to_string(),
                description: "Deploy application".to_string(),
                args: vec![
                    CommandArg {
                        name: "image".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "logs".to_string(),
                description: "Pod logs".to_string(),
                args: vec![
                    CommandArg {
                        name: "pod".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "scale".to_string(),
                description: "Scale deployment".to_string(),
                args: vec![
                    CommandArg {
                        name: "deployment".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                    CommandArg {
                        name: "replicas".to_string(),
                        arg_type: "number".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
        ],
        hooks: vec![],
        enabled: true,
    };

    static ref AI_PLUGIN: Plugin = Plugin {
        name: "ai".to_string(),
        version: "1.0.0".to_string(),
        description: "AI-powered features plugin".to_string(),
        author: "DevUtils Team".to_string(),
        commands: vec![
            PluginCommand {
                name: "generate".to_string(),
                description: "Generate code".to_string(),
                args: vec![
                    CommandArg {
                        name: "prompt".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "explain".to_string(),
                description: "Explain code".to_string(),
                args: vec![
                    CommandArg {
                        name: "code".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "test".to_string(),
                description: "Generate tests".to_string(),
                args: vec![
                    CommandArg {
                        name: "code".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "refactor".to_string(),
                description: "Refactor code".to_string(),
                args: vec![
                    CommandArg {
                        name: "code".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                    CommandArg {
                        name: "pattern".to_string(),
                        arg_type: "string".to_string(),
                        required: false,
                        default: None,
                    },
                ],
            },
            PluginCommand {
                name: "complete".to_string(),
                description: "Code completion".to_string(),
                args: vec![
                    CommandArg {
                        name: "code".to_string(),
                        arg_type: "string".to_string(),
                        required: true,
                        default: None,
                    },
                ],
            },
        ],
        hooks: vec![],
        enabled: true,
    };
}
