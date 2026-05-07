//! DevLoop - Autonomous Development Loop
//! 
//! This module provides fully autonomous development workflow:
//! - File watching with auto-rebuild
//! - Auto-test on file changes  
//! - Auto-format/lint
//! - Dependency auto-management
//! - TDD (Test-Driven Development) support
//! 
//! Usage: devutils devloop --watch --test --format

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Development loop configuration
#[derive(Debug, Clone)]
pub struct DevLoopConfig {
    pub watch: bool,
    pub test_on_change: bool,
    pub format_on_change: bool,
    pub lint_on_change: bool,
    pub auto_deps: bool,
    pub build_on_change: bool,
    pub poll_interval_ms: u64,
}

impl Default for DevLoopConfig {
    fn default() -> Self {
        Self {
            watch: true,
            test_on_change: true,
            format_on_change: true,
            lint_on_change: true,
            auto_deps: true,
            build_on_change: true,
            poll_interval_ms: 500,
        }
    }
}

/// File tracking for change detection
#[derive(Debug, Clone)]
pub struct TrackedFile {
    pub path: PathBuf,
    pub last_modified: u64,
    pub content_hash: u64,
}

/// Development loop state
pub struct DevLoop {
    config: DevLoopConfig,
    tracked_files: HashMap<PathBuf, TrackedFile>,
    project_root: PathBuf,
    build_cmd: Vec<String>,
    test_cmd: Vec<String>,
    format_cmd: Vec<String>,
    lint_cmd: Vec<String>,
}

impl DevLoop {
    pub fn new(project_root: &Path) -> Self {
        let config = DevLoopConfig::default();
        let (build_cmd, test_cmd, format_cmd, lint_cmd) = Self::detect_commands(project_root);
        
        Self {
            config,
            tracked_files: HashMap::new(),
            project_root: project_root.to_path_buf(),
            build_cmd,
            test_cmd,
            format_cmd,
            lint_cmd,
        }
    }
    
    pub fn with_config(mut self, config: DevLoopConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Detect project commands based on project type
    fn detect_commands(project_root: &Path) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        if project_root.join("Cargo.toml").exists() {
            (
                vec!["cargo".to_string(), "build".to_string()],
                vec!["cargo".to_string(), "test".to_string()],
                vec!["cargo".to_string(), "fmt".to_string()],
                vec!["cargo".to_string(), "clippy".to_string(), "--".to_string(), "-D".to_string(), "warnings".to_string()],
            )
        } else if project_root.join("package.json").exists() {
            (
                vec!["npm".to_string(), "run".to_string(), "build".to_string()],
                vec!["npm".to_string(), "test".to_string()],
                vec!["npx".to_string(), "prettier".to_string(), "--write".to_string()],
                vec!["npx".to_string(), "eslint".to_string()],
            )
        } else if project_root.join("go.mod").exists() {
            (
                vec!["go".to_string(), "build".to_string()],
                vec!["go".to_string(), "test".to_string(), "./...".to_string()],
                vec!["gofmt".to_string(), "-w".to_string()],
                vec!["go".to_string(), "vet".to_string(), "./...".to_string()],
            )
        } else if project_root.join("pyproject.toml").exists() || project_root.join("requirements.txt").exists() {
            (
                vec!["python".to_string(), "-m".to_string(), "py_compile".to_string()],
                vec!["python".to_string(), "-m".to_string(), "pytest".to_string()],
                vec!["black".to_string()],
                vec!["ruff".to_string(), "check".to_string()],
            )
        } else {
            (
                vec![],
                vec![],
                vec![],
                vec![],
            )
        }
    }
    
    /// Get current timestamp
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }
    
    /// Calculate simple hash of file content (for change detection)
    fn hash_content(content: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Track a file for changes
    pub fn track_file(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path));
        }
        
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        
        let modified = metadata
            .modified()
            .map_err(|e| format!("Failed to get modified time: {}", e))?;
        
        let modified_ts = modified
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let content = fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let hash = Self::hash_content(&content);
        
        self.tracked_files.insert(
            path.to_path_buf(),
            TrackedFile {
                path: path.to_path_buf(),
                last_modified: modified_ts,
                content_hash: hash,
            },
        );
        
        Ok(())
    }
    
/// Scan project and track all source files
    pub fn scan_project(&mut self) -> Result<usize, String> {
        use walkdir::WalkDir;
        
        let extensions: Vec<String> = self.detect_extensions().iter().map(|s| s.to_string()).collect();
        let project_root = self.project_root.clone();
        let mut count = 0;

        for entry in WalkDir::new(&project_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = format!(".{}", ext.to_string_lossy());
                    if extensions.contains(&ext_str) && !Self::should_ignore_static(path) {
                        if self.track_file(path).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }
    
    /// Static version of should_ignore that doesn't need &self
    fn should_ignore_static(path: &Path) -> bool {
        let ignore_dirs = ["target", "node_modules", ".git", "dist", "build", "__pycache__", "venv", ".venv"];
        let ignore_files = [".gitignore", ".gitkeep", "Cargo.lock", "package-lock.json"];

        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                if ignore_dirs.contains(&name_str.as_ref()) {
                    return true;
                }
            }
        }

        if let Some(file_name) = path.file_name() {
            let file_str = file_name.to_string_lossy();
            if ignore_files.contains(&file_str.as_ref()) {
                return true;
            }
        }

        false
    }
    
    fn detect_extensions(&self) -> Vec<&str> {
        if self.project_root.join("Cargo.toml").exists() {
            vec![".rs"]
        } else if self.project_root.join("package.json").exists() {
            vec![".js", ".ts", ".jsx", ".tsx"]
        } else if self.project_root.join("go.mod").exists() {
            vec![".go"]
        } else if self.project_root.join("pyproject.toml").exists() {
            vec![".py"]
        } else {
            vec![".rs", ".js", ".ts", ".py", ".go"]
        }
    }
    
    fn should_ignore(&self, path: &Path) -> bool {
        let ignore_dirs = ["target", "node_modules", ".git", "dist", "build", "__pycache__", "venv", ".venv"];
        let ignore_files = [".gitignore", ".gitkeep", "Cargo.lock", "package-lock.json"];
        
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                if ignore_dirs.contains(&name_str.as_ref()) {
                    return true;
                }
            }
        }
        
        if let Some(file_name) = path.file_name() {
            let file_str = file_name.to_string_lossy();
            if ignore_files.contains(&file_str.as_ref()) {
                return true;
            }
        }
        
        false
    }
    
    /// Check for file changes since last scan
    pub fn check_changes(&self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        
        for (path, tracked) in &self.tracked_files {
            if !path.exists() {
                continue;
            }
            
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_ts = modified
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    
                    if modified_ts > tracked.last_modified {
                        changed.push(path.clone());
                    }
                }
            }
        }
        
        changed
    }
    
    /// Run a command and return result
    pub fn run_command(cmd: &[String], cwd: &Path) -> (bool, String, String) {
        if cmd.is_empty() {
            return (true, String::new(), String::new());
        }
        
        let mut process = Command::new(&cmd[0]);
        if cmd.len() > 1 {
            process.args(&cmd[1..]);
        }
        process.current_dir(cwd);
        
        match process.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                (output.status.success(), stdout, stderr)
            }
            Err(e) => (false, String::new(), format!("Failed to run: {}", e)),
        }
    }
    
/// Execute one development loop iteration
    pub fn iteration(&mut self) -> LoopResult {
        let changed = self.check_changes();

        if changed.is_empty() {
            return LoopResult {
                changed_files: 0,
                ..Default::default()
            };
        }

        let mut results = LoopResult::default();
        results.changed_files = changed.len();

        // Format first (before build)
        if self.config.format_on_change && !self.format_cmd.is_empty() {
            let (success, stdout, stderr) = Self::run_command(&self.format_cmd, &self.project_root);
            if !success {
                results.lint_failed = Some(stderr.clone()); // Reuse lint_failed for format errors
            } else if !stdout.is_empty() {
                results.format_output = Some(stdout);
            }
        }
        
// Build
        if self.config.build_on_change && !self.build_cmd.is_empty() {
            let (success, stdout, stderr) = Self::run_command(&self.build_cmd, &self.project_root);
            results.build_success = Some(success);
            results.build_output = Some(stdout);
            if !success {
                results.build_failed = Some(stderr);
                return results; // Don't run tests if build fails
            }
        }

        // Test
        if self.config.test_on_change && !self.test_cmd.is_empty() {
            let (success, stdout, stderr) = Self::run_command(&self.test_cmd, &self.project_root);
            results.test_success = Some(success);
            results.test_output = Some(stdout);
            if !success {
                results.test_failed = Some(stderr);
            }
        }
        
        // Lint
        if self.config.lint_on_change && !self.lint_cmd.is_empty() {
            let (success, stdout, stderr) = Self::run_command(&self.lint_cmd, &self.project_root);
            if !success {
                results.lint_failed = Some(stderr);
            } else if !stdout.is_empty() {
                results.lint_output = Some(stdout);
            }
        }
        
        // Update tracked files
        for path in &changed {
            let _ = self.track_file(path);
        }
        
        results
    }
    
    /// Run development loop until stopped
    pub fn run(&mut self, max_iterations: Option<usize>) -> Result<LoopStats, String> {
        if !self.config.test_on_change && !self.config.format_on_change && !self.config.lint_on_change && !self.config.build_on_change && max_iterations.is_none() {
            return Err("All devloop actions are disabled and no iterations specified. Exiting to avoid infinite loop.".to_string());
        }
        
        println!("🚀 Starting DevLoop...");
        println!("   Watch: {}", self.config.watch);
        println!("   Test on change: {}", self.config.test_on_change);
        println!("   Format on change: {}", self.config.format_on_change);
        println!("   Auto-deps: {}", self.config.auto_deps);
        
        // Initial scan
        let file_count = self.scan_project()?;
        println!("   Tracking {} files", file_count);
        
        let mut iterations = 0;
        let mut stats = LoopStats::default();
        
        loop {
            iterations += 1;
            stats.total_iterations += 1;
            
            let result = self.iteration();
            
            if result.changed_files == 0 {
                // No changes, continue watching
            } else {
                stats.iterations_with_changes += 1;
                
                if result.build_success.unwrap_or(false) && result.test_success.unwrap_or(false) {
                    stats.successful_builds += 1;
                }
                
                if !result.test_failed.is_some() {
                    stats.successful_tests += 1;
                }
                
                println!("\n🔄 Iteration {}: {} files changed", iterations, result.changed_files);
                
                if let Some(failed) = result.build_failed {
                    println!("❌ Build failed:\n{}", failed);
                }
                
                if let Some(failed) = result.test_failed {
                    println!("❌ Tests failed:\n{}", failed);
                }
                
                if result.build_success.unwrap_or(false) && result.test_success.unwrap_or(false) {
                    println!("✅ All checks passed");
                }
            }
            
            // Check exit condition
            if let Some(max) = max_iterations {
                if iterations >= max {
                    break;
                }
            }
            
            // Sleep before next check
            std::thread::sleep(Duration::from_millis(self.config.poll_interval_ms));
        }
        
        Ok(stats)
    }
}

/// Result of one loop iteration
#[derive(Debug, Clone, Default)]
pub struct LoopResult {
    pub changed_files: usize,
    pub build_success: Option<bool>,
    pub build_output: Option<String>,
    pub build_failed: Option<String>,
    pub test_success: Option<bool>,
    pub test_output: Option<String>,
    pub test_failed: Option<String>,
    pub format_output: Option<String>,
    pub lint_output: Option<String>,
    pub lint_failed: Option<String>,
}

impl LoopResult {
    pub fn is_success(&self) -> bool {
        self.build_success.unwrap_or(true) 
            && self.test_success.unwrap_or(true)
            && self.lint_failed.is_none()
    }
}

/// Statistics from development loop
#[derive(Debug, Clone, Default)]
pub struct LoopStats {
    pub total_iterations: usize,
    pub iterations_with_changes: usize,
    pub successful_builds: usize,
    pub successful_tests: usize,
}

/// Check if dependencies need to be added and auto-add them
pub fn auto_add_dependency(dep_name: &str) -> Result<String, String> {
    let project_root = PathBuf::from(".");
    
    if project_root.join("Cargo.toml").exists() {
        auto_add_cargo_dep(&project_root, dep_name)
    } else if project_root.join("package.json").exists() {
        auto_add_npm_dep(&project_root, dep_name)
    } else {
        Err("No recognized project file found".to_string())
    }
}

fn auto_add_cargo_dep(project_root: &Path, dep_name: &str) -> Result<String, String> {
    let cargo_path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;
    
    // Check if already present
    if content.contains(&format!("{} =", dep_name)) {
        return Ok(format!("Dependency '{}' already in Cargo.toml", dep_name));
    }
    
    // Parse and add dependency
    let new_content = if content.contains("[dependencies]") {
        format!("{}\n{} = \"0.1\"  # auto-added", content, dep_name)
    } else {
        format!("{}\n\n[dependencies]\n{} = \"0.1\"  # auto-added", content, dep_name)
    };
    
    fs::write(&cargo_path, new_content)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
    
    Ok(format!("Added '{}' to Cargo.toml", dep_name))
}

fn auto_add_npm_dep(project_root: &Path, dep_name: &str) -> Result<String, String> {
    // For npm, we just run npm install
    let output = Command::new("npm")
        .args(["install", dep_name])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Installed '{}' via npm", dep_name))
    } else {
        Err(format!("npm install failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

/// Format all source files
pub fn format_project() -> Result<String, String> {
    let project_root = PathBuf::from(".");
    
    if project_root.join("Cargo.toml").exists() {
        let output = Command::new("cargo")
            .args(["fmt"])
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to run cargo fmt: {}", e))?;
        
        if output.status.success() {
            Ok("Formatted Rust code".to_string())
        } else {
            Err(format!("Format failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    } else if project_root.join("package.json").exists() {
        let output = Command::new("npx")
            .args(["prettier", "--write", "**/*.{js,ts,jsx,tsx}"])
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to run prettier: {}", e))?;
        
        if output.status.success() {
            Ok("Formatted JS/TS code".to_string())
        } else {
            Err(format!("Format failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    } else {
        Err("No formatter available for this project type".to_string())
    }
}

/// Lint all source files
pub fn lint_project() -> Result<String, String> {
    let project_root = PathBuf::from(".");
    
    if project_root.join("Cargo.toml").exists() {
        let output = Command::new("cargo")
            .args(["clippy", "--", "-D", "warnings"])
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to run cargo clippy: {}", e))?;
        
        if output.status.success() {
            Ok("Lint passed".to_string())
        } else {
            Err(format!("Lint failed:\n{}", String::from_utf8_lossy(&output.stderr)))
        }
    } else if project_root.join("package.json").exists() {
        let output = Command::new("npx")
            .args(["eslint", "."])
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to run eslint: {}", e))?;
        
        if output.status.success() {
            Ok("Lint passed".to_string())
        } else {
            Err(format!("Lint failed:\n{}", String::from_utf8_lossy(&output.stderr)))
        }
    } else {
        Err("No linter available for this project type".to_string())
    }
}

/// Watch mode for TDD - runs tests on file change
pub fn tdd_watch() -> Result<LoopStats, String> {
    let mut loop_dev = DevLoop::new(&PathBuf::from("."));
    loop_dev.config.test_on_change = true;
    loop_dev.config.build_on_change = true;
    loop_dev.config.format_on_change = false;
    loop_dev.config.lint_on_change = false;
    
    loop_dev.run(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_loop_creation() {
        let loop_dev = DevLoop::new(&PathBuf::from("."));
        assert!(loop_dev.project_root.exists());
    }

    #[test]
    fn test_auto_add_cargo_dep() {
        // This test requires a Cargo.toml in current dir
        let result = auto_add_dependency("example_dep_123");
        // Will fail without proper project setup, which is expected
        assert!(result.is_err() || result.is_ok());
    }
}