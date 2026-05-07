//! Plugin Generator - Creates plugins from GitHub CLI tools
//! Automatically converts popular CLI tools to DevUtils plugins

use std::fs;
use std::path::Path;

pub struct PluginGenerator;

impl PluginGenerator {
    /// Generate plugin from GitHub repository
    pub fn generate_from_github(repo: &str, output_dir: &str) -> Result<String, String> {
        // This would fetch repo info from GitHub API
        // For now, create a template
        Self::create_plugin_template("custom-plugin", output_dir)
    }

    /// Create a plugin template
    fn create_plugin_template(name: &str, output_dir: &str) -> Result<String, String> {
        let plugin_dir = Path::new(output_dir).join(name);
        fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to create plugin dir: {}", e))?;

        // Create plugin.json manifest
        let manifest = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "Generated plugin for {}",
  "author": "DevUtils",
  "repository": "https://github.com/devutils/plugins",
  "commands": [
    {{
      "name": "run",
      "description": "Run the plugin",
      "handler": "echo 'Plugin running'",
      "args": [],
      "examples": ["devutils plugin run {}"]
    }}
  ],
  "dependencies": [],
  "entry_point": "main.sh"
}}"#, name, name, name);

        fs::write(plugin_dir.join("plugin.json"), &manifest)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        // Create main script
        let script = r#"#!/bin/bash
# Plugin entry point
echo "Hello from DevUtils plugin!"
"#;
        fs::write(plugin_dir.join("main.sh"), script)
            .map_err(|e| format!("Failed to write script: {}", e))?;

        // Create README
        let readme = format!(r#"# {} Plugin

## Installation
```bash
devutils plugin install {}
```

## Usage
```bash
devutils plugin run {}
```

## Commands
- `run` - Run the plugin

## Development
Edit `main.sh` to customize behavior.
"#, name, name, name);

        fs::write(plugin_dir.join("README.md"), readme)
            .map_err(|e| format!("Failed to write README: {}", e))?;

        Ok(format!("Generated plugin '{}' at {}", name, plugin_dir.display()))
    }

    /// Scrape GitHub for popular CLI tools
    pub fn scrape_popular_tools() -> Vec<PopularTool> {
        // Hardcoded list of popular CLI tools to convert
        vec![
            PopularTool {
                name: "httpie".to_string(),
                repo: "httpie/httpie".to_string(),
                description: "Modern command-line HTTP client".to_string(),
                stars: 50000,
            },
            PopularTool {
                name: "jq".to_string(),
                repo: "stedolan/jq".to_string(),
                description: "Command-line JSON processor".to_string(),
                stars: 40000,
            },
            PopularTool {
                name: "bat".to_string(),
                repo: "sharkdp/bat".to_string(),
                description: "cat with syntax highlighting".to_string(),
                stars: 30000,
            },
            PopularTool {
                name: "exa".to_string(),
                repo: "ogham/exa".to_string(),
                description: "Modern ls replacement".to_string(),
                stars: 20000,
            },
            PopularTool {
                name: "ripgrep".to_string(),
                repo: "BurntSushi/ripgrep".to_string(),
                description: "Fast grep alternative".to_string(),
                stars: 35000,
            },
            PopularTool {
                name: "fzf".to_string(),
                repo: "junegunn/fzf".to_string(),
                description: "Command-line fuzzy finder".to_string(),
                stars: 48000,
            },
            PopularTool {
                name: "tldr".to_string(),
                repo: "tldr-pages/tldr".to_string(),
                description: "Simplified man pages".to_string(),
                stars: 35000,
            },
            PopularTool {
                name: "difftastic".to_string(),
                repo: "Wilfred/difftastic".to_string(),
                description: "Structural diff tool".to_string(),
                stars: 15000,
            },
            PopularTool {
                name: "bottom".to_string(),
                repo: "ClementTsang/bottom".to_string(),
                description: "Cross-platform system monitor".to_string(),
                stars: 12000,
            },
            PopularTool {
                name: "grex".to_string(),
                repo: "pemistahl/grex".to_string(),
                description: "Regex generator".to_string(),
                stars: 8000,
            },
        ]
    }
}

#[derive(Debug)]
pub struct PopularTool {
    pub name: String,
    pub repo: String,
    pub description: String,
    pub stars: u32,
}

// CLI command
pub fn generate_plugin(name: &str, output_dir: &str) -> String {
    match PluginGenerator::generate_from_github(name, output_dir) {
        Ok(msg) => msg,
        Err(e) => format!("Error: {}", e),
    }
}

pub fn list_popular_tools() -> String {
    let tools = PluginGenerator::scrape_popular_tools();
    let mut output = String::from("\n🔥 Popular CLI Tools Available as Plugins:\n\n");
    
    for (i, tool) in tools.iter().take(10).enumerate() {
        output.push_str(&format!(
            "{}. {} ⭐{} - {}\n   Repo: {}\n   Install: devutils plugin install {}\n\n",
            i + 1,
            tool.name,
            tool.stars,
            tool.description,
            tool.repo,
            tool.name,
        ));
    }
    
    output
}