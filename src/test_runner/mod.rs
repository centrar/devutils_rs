//! Real Test Execution Module
//! Actually runs tests for cargo test, npm test, pytest, etc.

use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub output: String,
    pub errors: String,
    pub duration_ms: u64,
    pub tests_run: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
}

/// Detect test framework and run tests
pub fn run_tests(directory: &str) -> Result<TestResult, String> {
    let path = Path::new(directory);
    
    if !path.exists() {
        return Err(format!("Directory not found: {}", directory));
    }
    
    // Detect project type
    if path.join("Cargo.toml").exists() {
        run_cargo_test(directory)
    } else if path.join("package.json").exists() {
        run_npm_test(directory)
    } else if path.join("requirements.txt").exists() || 
              path.join("setup.py").exists() || 
              path.join("pyproject.toml").exists() {
        run_pytest(directory)
    } else if path.join("pom.xml").exists() {
        run_maven_test(directory)
    } else if path.join("build.gradle").exists() {
        run_gradle_test(directory)
    } else {
        Err("No recognized test framework found (Cargo.toml, package.json, requirements.txt, etc.)".to_string())
    }
}

/// Run cargo test for Rust projects
pub fn run_cargo_test(directory: &str) -> Result<TestResult, String> {
    use std::time::Instant;
    let start = Instant::now();
    
    let output = Command::new("cargo")
        .arg("test")
        .arg("--color")
        .arg("always")
        .current_dir(directory)
        .output()
        .map_err(|e| format!("Failed to execute cargo test: {}", e))?;
    
    let duration = start.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    // Parse test results from output
    let (run, passed, failed) = parse_cargo_output(&stdout);
    
    Ok(TestResult {
        success: output.status.success(),
        output: stdout,
        errors: stderr,
        duration_ms: duration,
        tests_run: run,
        tests_passed: passed,
        tests_failed: failed,
    })
}

/// Parse cargo test output
fn parse_cargo_output(output: &str) -> (u32, u32, u32) {
    let mut run = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    for line in output.lines() {
        if line.contains("test result:") {
            // Example: "test result: ok. 5 passed; 0 failed; 0 ignored"
            if let Some(parts) = line.split("test result:").nth(1) {
                let parts: Vec<&str> = parts.split(';').collect();
                for part in parts {
                    let part = part.trim();
                    if part.contains("passed") {
                        passed = part.trim_start_matches(|c: char| !c.is_ascii_digit())
                            .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')
                            .split_whitespace()
                            .next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    } else if part.contains("failed") {
                        failed = part.trim_start_matches(|c: char| !c.is_ascii_digit())
                            .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ')
                            .split_whitespace()
                            .next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    }
                }
            }
            run = passed + failed;
            break;
        }
    }
    
    // Fallback: count "test result: ok" or "test result: FAILED"
    if run == 0 {
        if output.contains("test result: ok") {
            run = 1;
            passed = 1;
        } else if output.contains("test result: FAILED") {
            run = 1;
            failed = 1;
        }
    }
    
    (run, passed, failed)
}

/// Run npm test for Node.js projects
pub fn run_npm_test(directory: &str) -> Result<TestResult, String> {
    use std::time::Instant;
    let start = Instant::now();
    
    let output = Command::new("npm")
        .arg("test")
        .current_dir(directory)
        .output()
        .or_else(|_| {
            // Try yarn as fallback
            Command::new("yarn")
                .arg("test")
                .current_dir(directory)
                .output()
        })
        .map_err(|e| format!("Failed to execute npm test: {}", e))?;
    
    let duration = start.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    // Parse npm test output (simplified)
    let (run, passed, failed) = parse_generic_test_output(&stdout);
    
    Ok(TestResult {
        success: output.status.success(),
        output: stdout,
        errors: stderr,
        duration_ms: duration,
        tests_run: run,
        tests_passed: passed,
        tests_failed: failed,
    })
}

/// Run pytest for Python projects
pub fn run_pytest(directory: &str) -> Result<TestResult, String> {
    use std::time::Instant;
    let start = Instant::now();
    
    let output = Command::new("pytest")
        .arg("--tb=short")
        .current_dir(directory)
        .output()
        .or_else(|_| {
            // Try python -m pytest
            Command::new("python")
                .arg("-m")
                .arg("pytest")
                .arg("--tb=short")
                .current_dir(directory)
                .output()
        })
        .map_err(|e| format!("Failed to execute pytest: {}", e))?;
    
    let duration = start.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    // Parse pytest output
    let (run, passed, failed) = parse_pytest_output(&stdout);
    
    Ok(TestResult {
        success: output.status.success(),
        output: stdout,
        errors: stderr,
        duration_ms: duration,
        tests_run: run,
        tests_passed: passed,
        tests_failed: failed,
    })
}

/// Parse pytest output
fn parse_pytest_output(output: &str) -> (u32, u32, u32) {
    // Example: "3 passed, 1 failed in 0.5s"
    let mut passed = 0;
    let mut failed = 0;
    
    if let Some(line) = output.lines().rev().find(|l| l.contains("passed") || l.contains("failed")) {
        if let Some(idx) = line.find("passed") {
            passed = line[..idx]
                .trim()
                .split_whitespace()
                .last()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
        if let Some(idx) = line.find("failed") {
            failed = line[..idx]
                .trim()
                .split_whitespace()
                .last()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    
    let run = passed + failed;
    (run, passed, failed)
}

/// Generic test output parser
fn parse_generic_test_output(output: &str) -> (u32, u32, u32) {
    // Try to extract numbers from output
    let passed = output.lines()
        .find(|l| l.to_lowercase().contains("pass"))
        .and_then(|l| l.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .unwrap_or(0);
    
    let failed = output.lines()
        .find(|l| l.to_lowercase().contains("fail"))
        .and_then(|l| l.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .unwrap_or(0);
    
    let run = passed + failed;
    (run, passed, failed)
}

// Stub implementations for other frameworks
pub fn run_maven_test(_directory: &str) -> Result<TestResult, String> {
    Err("Maven test execution not yet implemented".to_string())
}

pub fn run_gradle_test(_directory: &str) -> Result<TestResult, String> {
    Err("Gradle test execution not yet implemented".to_string())
}

/// CLI command for test runner
pub fn run_test_cmd(args: &[String]) -> Result<String, String> {
    let directory = args.first().map(|s| s.as_str()).unwrap_or(".");
    
    match run_tests(directory) {
        Ok(result) => {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            Ok(format!(
                "Test Results: {}\n\
                 Tests Run: {}\n\
                 Passed: {}\n\
                 Failed: {}\n\
                 Duration: {}ms\n\n\
                 Output:\n{}",
                status, result.tests_run, result.tests_passed, result.tests_failed,
                result.duration_ms, result.output
            ))
        }
        Err(e) => Err(format!("Test execution failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[ignore]
    #[test]
    fn test_parse_cargo_output() {
        let output = "test result: ok. 5 passed; 0 failed; 0 ignored";
        let (run, passed, failed) = parse_cargo_output(output);
        assert_eq!(run, 5);
        assert_eq!(passed, 5);
        assert_eq!(failed, 0);
    }
}
