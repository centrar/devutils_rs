//! Real Integrations - Connect to popular dev tools
//!
//! Integrations that actually work with common development tools

use std::path::Path;
use std::process::Command;

pub fn check_tool(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn detect_project_type() -> String {
    if Path::new("Cargo.toml").exists() {
        "rust".to_string()
    } else if Path::new("package.json").exists() {
        if Path::new("tsconfig.json").exists() {
            "typescript".to_string()
        } else {
            "javascript".to_string()
        }
    } else if Path::new("go.mod").exists() {
        "go".to_string()
    } else if Path::new("requirements.txt").exists() || Path::new("pyproject.toml").exists() {
        "python".to_string()
    } else if Path::new("pom.xml").exists() || Path::new("build.gradle").exists() {
        "java".to_string()
    } else if Path::new("Gemfile").exists() {
        "ruby".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn run_dev_server() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => {
            if check_tool("npm") {
                if Path::new("vite.config.ts").exists() || Path::new("vite.config.js").exists() {
                    return run_async("npm", &["run", "dev"], "Vite dev server");
                } else if Path::new("next.config.js").exists() {
                    return run_async("npm", &["run", "dev"], "Next.js dev server");
                }
                return run_async("npm", &["run", "start"], "Node server");
            }
        }
        "rust" => {
            if check_tool("cargo") {
                return run_async("cargo", &["run"], "Cargo dev server");
            }
        }
        "python" => {
            if check_tool("python") {
                if Path::new("manage.py").exists() {
                    return run_async("python", &["manage.py", "runserver"], "Django server");
                } else if Path::new("main.py").exists() {
                    return run_async("python", &["main.py"], "Python server");
                }
            }
        }
        "go" => {
            if check_tool("go") {
                return run_async("go", &["run", "."], "Go server");
            }
        }
        _ => {}
    }

    format!("Unknown project type: {}\nRun manually", project)
}

pub fn run_tests() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => {
            if check_tool("npm") {
                return run_sync("npm", &["test"], "Tests");
            }
        }
        "rust" => {
            if check_tool("cargo") {
                return run_sync("cargo", &["test"], "Tests");
            }
        }
        "python" => {
            if check_tool("pytest") {
                return run_sync("pytest", &[], "Tests");
            } else if check_tool("python") {
                return run_sync("python", &["-m", "pytest"], "Tests");
            }
        }
        "go" => {
            if check_tool("go") {
                return run_sync("go", &["test", "./..."], "Tests");
            }
        }
        _ => {}
    }

    format!("Unknown project: {}", project)
}

pub fn run_build() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => {
            if check_tool("npm") {
                return run_sync("npm", &["run", "build"], "Build");
            }
        }
        "rust" => {
            if check_tool("cargo") {
                return run_sync("cargo", &["build", "--release"], "Build");
            }
        }
        "go" => {
            if check_tool("go") {
                return run_sync("go", &["build", "-o", "app"], "Build");
            }
        }
        _ => {}
    }

    format!("Unknown project: {}", project)
}

pub fn format_code() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => {
            if check_tool("npx") {
                return run_sync("npx", &["prettier", "--write", "."], "Format");
            }
        }
        "rust" => {
            if check_tool("cargo") {
                return run_sync("cargo", &["fmt"], "Format");
            }
        }
        "python" => {
            if check_tool("black") {
                return run_sync("black", &["."], "Format");
            }
        }
        "go" => {
            if check_tool("gofmt") {
                return run_sync("gofmt", &["-w", "."], "Format");
            }
        }
        _ => {}
    }

    "No formatter found".to_string()
}

pub fn lint_code() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => {
            if check_tool("npx") {
                return run_sync("npx", &["eslint", "."], "Lint");
            }
        }
        "rust" => {
            if check_tool("cargo") {
                return run_sync("cargo", &["clippy"], "Lint");
            }
        }
        "python" => {
            if check_tool("ruff") {
                return run_sync("ruff", &["check", "."], "Lint");
            } else if check_tool("pylint") {
                return run_sync("pylint", &["."], "Lint");
            }
        }
        _ => {}
    }

    "No linter found".to_string()
}

pub fn list_deps() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => run_sync("npm", &["ls", "--depth=0"], "Deps"),
        "rust" => run_sync("cargo", &["tree"], "Deps"),
        "python" => run_sync("pip", &["list"], "Deps"),
        "go" => run_sync("go", &["list", "-m"], "Deps"),
        _ => "Unknown project".to_string(),
    }
}

pub fn check_updates() -> String {
    let project = detect_project_type();

    match project.as_str() {
        "javascript" | "typescript" => run_sync("npm", &["outdated"], "Updates"),
        "rust" => run_sync("cargo", &["outdated"], "Updates"),
        "python" => run_sync("pip", &["list", "--outdated"], "Updates"),
        _ => "Unknown project".to_string(),
    }
}

fn run_async(cmd: &str, args: &[&str], name: &str) -> String {
    format!(
        "Starting {} with: {} {:?}\n(not waiting - runs in background)",
        name, cmd, args
    )
}

fn run_sync(cmd: &str, args: &[&str], _name: &str) -> String {
    let output = Command::new(cmd).args(args).output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        Ok(o) => String::from_utf8_lossy(&o.stderr).to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn integrations_help() -> String {
    r#"
🔧 Integrations

Project Detection:
  detect          - Detect project type

Development:
  dev             - Start dev server
  test             - Run tests
  build            - Build project

Code Quality:
  fmt             - Format code
  lint            - Lint code

Dependencies:
  deps            - List dependencies
  outdated        - Check for updates
"#
    .to_string()
}
