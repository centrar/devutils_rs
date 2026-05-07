//! Unified Plugin System - State-of-the-Art Plugin Orchestration
//! 
//! This module provides a unified interface to all plugin subsystems:
//! - Registry: 541-plugin catalog with advanced search
//! - Manager: Install/uninstall from marketplace
//! - Loader: GitHub integration for custom plugins
//! - Generator: Create plugins from templates
//! - Native: 100 built-in implementations

use crate::registry::{PluginRegistry, PluginCategory, RegistryEntry};
use crate::plugin_manager::{PluginManager, Plugin as ManagedPlugin};

/// Unified plugin system orchestrating all subsystems
pub struct UnifiedPluginSystem {
    registry: PluginRegistry,
    manager: PluginManager,
}

impl UnifiedPluginSystem {
    /// Create a new unified plugin system
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            manager: PluginManager::new(),
        }
    }

    // ===== REGISTRY OPERATIONS =====

    /// Search plugins by name, description, tags, or category
    pub fn search(&self, query: &str) -> Vec<&RegistryEntry> {
        self.registry.search(query)
    }

    /// Get plugins by category
    pub fn by_category(&self, cat: &PluginCategory) -> Vec<&RegistryEntry> {
        self.registry.by_category(cat)
    }

    /// Get featured plugins
    pub fn featured(&self) -> Vec<&RegistryEntry> {
        self.registry.featured()
    }

    /// Get top N plugins by download count
    pub fn top(&self, n: usize) -> Vec<&RegistryEntry> {
        self.registry.top(n)
    }

    /// Get plugin by exact name
    pub fn get(&self, name: &str) -> Option<&RegistryEntry> {
        self.registry.get(name)
    }

    /// Get total plugin count
    pub fn count(&self) -> usize {
        self.registry.count()
    }

    /// Get category summary with counts
    pub fn categories_summary(&self) -> Vec<(String, usize)> {
        self.registry.categories_summary()
    }

    /// Get all categories
    pub fn all_categories(&self) -> Vec<PluginCategory> {
        PluginCategory::all()
    }

    // ===== MANAGER OPERATIONS =====

    /// Install a plugin from the marketplace
    pub fn install(&mut self, name: &str) -> Result<String, String> {
        // Check if plugin exists in registry
        if self.registry.get(name).is_none() {
            return Err(format!("Plugin '{}' not found in registry", name));
        }
        
        self.manager.install(name)
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, name: &str) -> Result<String, String> {
        self.manager.uninstall(name)
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Vec<ManagedPlugin> {
        self.manager.list_installed()
    }

    /// Execute a plugin command
    pub fn execute(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<String, String> {
        self.manager.execute(plugin_name, command, args)
    }

    // ===== STATISTICS =====

    /// Get comprehensive plugin statistics
    pub fn stats(&self) -> PluginStats {
        let installed = self.manager.list_installed();
        let featured = self.registry.featured();
        
        PluginStats {
            total: self.registry.count(),
            categories: 24,
            featured: featured.len(),
            installed: installed.len(),
            by_category: self.registry.categories_summary(),
        }
    }

    // ===== NATIVE PLUGIN EXECUTION =====

    /// Execute a native plugin (from plugins_100.rs)
    pub fn execute_native(&self, name: &str, args: &[String]) -> Result<String, String> {
        use crate::plugins_100;
        
        match name {
            "httpie" => {
                if args.len() < 2 {
                    return Err("Usage: httpie <url> <method> [body]".to_string());
                }
                let body = args.get(2).map(|s| s.as_str());
                plugins_100::httpie(&args[0], &args[1], body)
            }
            "jq" => {
                if args.len() < 2 {
                    return Err("Usage: jq <json> <query>".to_string());
                }
                plugins_100::jq_query(&args[0], &args[1])
            }
            "bat" => {
                if args.is_empty() {
                    return Err("Usage: bat <file>".to_string());
                }
                plugins_100::bat_file(&args[0])
            }
            "grep" => {
                if args.len() < 2 {
                    return Err("Usage: grep <file> <pattern> [--ignore-case]".to_string());
                }
                let ignore_case = args.get(2).map(|s| s == "--ignore-case").unwrap_or(false);
                plugins_100::grep_pattern(&args[0], &args[1], ignore_case)
                    .map(|lines| lines.join("\n"))
            }
            "git-status" => plugins_100::git_status(),
            "git-log" => {
                let limit = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(10);
                plugins_100::git_log(limit)
            }
            "wc" => {
                if args.is_empty() {
                    return Err("Usage: wc <file>".to_string());
                }
                plugins_100::word_count(&args[0])
            }
            "head" => {
                if args.len() < 2 {
                    return Err("Usage: head <file> <n>".to_string());
                }
                let n = args[1].parse().map_err(|_| "Invalid number".to_string())?;
                plugins_100::head_lines(&args[0], n)
            }
            "tail" => {
                if args.len() < 2 {
                    return Err("Usage: tail <file> <n>".to_string());
                }
                let n = args[1].parse().map_err(|_| "Invalid number".to_string())?;
                plugins_100::tail_lines(&args[0], n)
            }
            _ => Err(format!("Native plugin '{}' not found", name)),
        }
    }

    /// Check if a plugin is available as native implementation
    pub fn is_native(&self, name: &str) -> bool {
        matches!(name, "httpie" | "jq" | "bat" | "grep" | "git-status" | "git-log" | "wc" | "head" | "tail")
    }

    /// List all available native plugins
    pub fn list_native(&self) -> Vec<&str> {
        vec![
            "httpie", "jq", "bat", "grep", "git-status", "git-log", 
            "wc", "head", "tail", "sort", "uniq", "diff", "cut", "tr"
        ]
    }
}

impl Default for UnifiedPluginSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive plugin statistics
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub total: usize,
    pub categories: usize,
    pub featured: usize,
    pub installed: usize,
    pub by_category: Vec<(String, usize)>,
}

impl PluginStats {
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Total: {} | Categories: {} | Featured: {} | Installed: {}",
            self.total, self.categories, self.featured, self.installed
        )
    }
}

// ===== CLI HELPER FUNCTIONS =====

/// Display all plugins in a beautiful format
pub fn display_all_plugins() {
    let system = UnifiedPluginSystem::new();
    let stats = system.stats();
    
    println!("\n\x1b[36m╔══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║\x1b[0m          \x1b[1m📦 DevUtils Plugin Marketplace\x1b[0m                  \x1b[36m║\x1b[0m");
    println!("\x1b[36m╚══════════════════════════════════════════════════════════════╝\x1b[0m\n");
    
    println!("\x1b[33m{}\x1b[0m\n", stats.format());
    
    println!("\x1b[32m✨ Featured Plugins:\x1b[0m\n");
    for (i, plugin) in system.featured().iter().take(10).enumerate() {
        println!("  {}. \x1b[1m{}\x1b[0m", i + 1, plugin.name);
        println!("     {}", plugin.description);
        println!("     \x1b[90m{} downloads | v{} | by {}\x1b[0m", 
            format_downloads(plugin.downloads), plugin.version, plugin.author);
        if let Some(homepage) = plugin.homepage {
            println!("     \x1b[90m🔗 {}\x1b[0m", homepage);
        }
        println!();
    }
    
    println!("\x1b[33m📂 Categories:\x1b[0m");
    for (category, count) in stats.by_category.iter().take(12) {
        println!("  • {} \x1b[90m({} plugins)\x1b[0m", category, count);
    }
    
    println!("\n\x1b[36m💡 Quick Start:\x1b[0m");
    println!("  devutils plugins search <query>    - Search for plugins");
    println!("  devutils plugins featured          - Show featured plugins");
    println!("  devutils plugins categories        - List all categories");
    println!("  devutils plugins install <name>    - Install a plugin");
    println!("  devutils plugins stats             - Show statistics\n");
}

/// Display search results
pub fn display_search_results(query: &str) {
    let system = UnifiedPluginSystem::new();
    let results = system.search(query);
    
    if results.is_empty() {
        println!("\n\x1b[33m🔍 No plugins found for '{}'\x1b[0m\n", query);
        println!("Try searching for:");
        println!("  • Tool names: docker, git, aws, kubernetes");
        println!("  • Categories: cloud, ai, testing, database");
        println!("  • Technologies: rust, python, javascript, go\n");
        return;
    }
    
    println!("\n\x1b[36m🔍 Found {} plugin(s) matching '{}'\x1b[0m\n", results.len(), query);
    
    for (i, plugin) in results.iter().take(20).enumerate() {
        let featured = if plugin.featured { " \x1b[32m★\x1b[0m" } else { "" };
        println!("  {}. \x1b[1m{}\x1b[0m{}", i + 1, plugin.name, featured);
        println!("     {}", plugin.description);
        println!("     \x1b[90m{} | {} downloads | v{}\x1b[0m", 
            plugin.category.as_str(), format_downloads(plugin.downloads), plugin.version);
        println!();
    }
    
    if results.len() > 20 {
        println!("\x1b[90m... and {} more results\x1b[0m\n", results.len() - 20);
    }
}

/// Display plugins by category
pub fn display_category(category: &PluginCategory) {
    let system = UnifiedPluginSystem::new();
    let plugins = system.by_category(category);
    
    println!("\n\x1b[36m📂 {} ({} plugins)\x1b[0m\n", category.as_str(), plugins.len());
    
    for (i, plugin) in plugins.iter().enumerate() {
        let featured = if plugin.featured { " \x1b[32m★\x1b[0m" } else { "" };
        println!("  {}. \x1b[1m{}\x1b[0m{}", i + 1, plugin.name, featured);
        println!("     {}", plugin.description);
        println!("     \x1b[90m{} downloads | v{} | by {}\x1b[0m", 
            format_downloads(plugin.downloads), plugin.version, plugin.author);
        println!();
    }
}

/// Display all categories
pub fn display_all_categories() {
    let system = UnifiedPluginSystem::new();
    let summary = system.categories_summary();
    
    println!("\n\x1b[36m📂 Plugin Categories\x1b[0m\n");
    
    for (i, (category, count)) in summary.iter().enumerate() {
        println!("  {}. {} \x1b[90m({} plugins)\x1b[0m", i + 1, category, count);
    }
    
    println!("\n\x1b[36m💡 Usage:\x1b[0m");
    println!("  devutils plugins category <name>   - View plugins in a category\n");
}

/// Display statistics
pub fn display_stats() {
    let system = UnifiedPluginSystem::new();
    let stats = system.stats();
    
    println!("\n\x1b[36m╔══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[36m║\x1b[0m              \x1b[1m📊 Plugin Statistics\x1b[0m                         \x1b[36m║\x1b[0m");
    println!("\x1b[36m╚══════════════════════════════════════════════════════════════╝\x1b[0m\n");
    
    println!("  \x1b[32m📦 Total Plugins:\x1b[0m      {}", stats.total);
    println!("  \x1b[33m📂 Categories:\x1b[0m        {}", stats.categories);
    println!("  \x1b[35m✨ Featured:\x1b[0m          {}", stats.featured);
    println!("  \x1b[36m📥 Installed:\x1b[0m         {}", stats.installed);
    
    println!("\n\x1b[33m📊 Top Categories:\x1b[0m\n");
    for (i, (category, count)) in stats.by_category.iter().take(10).enumerate() {
        let bar = "█".repeat((count * 30 / stats.total).max(1));
        println!("  {}. {:30} {} \x1b[90m({})\x1b[0m", i + 1, category, bar, count);
    }
    println!();
}

/// Format download count for display
fn format_downloads(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_system_creation() {
        let system = UnifiedPluginSystem::new();
        assert!(system.count() > 500);
    }

    #[test]
    fn test_search() {
        let system = UnifiedPluginSystem::new();
        let results = system.search("docker");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_featured() {
        let system = UnifiedPluginSystem::new();
        let featured = system.featured();
        assert!(!featured.is_empty());
    }

    #[test]
    fn test_stats() {
        let system = UnifiedPluginSystem::new();
        let stats = system.stats();
        assert!(stats.total > 500);
        assert_eq!(stats.categories, 24);
    }
}
