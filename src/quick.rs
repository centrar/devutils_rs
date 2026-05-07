//! Quick Commands - Shortcuts for common tasks
//!
//! Fast one-liner commands for everyday development

use std::process::Command;

pub fn run_quick_command(cmd: &str) -> String {
    if cmd.is_empty() {
        return quick_help();
    }

    match cmd {
        "g" | "git" => git_shortcut(),
        "npm" => npm_shortcut(),
        "d" | "docker" => docker_shortcut(),
        "ls" | "dir" => list_files(),
        "new" | "init" => {
            let ctx = crate::project::ProjectContext::detect();
            init_project(&ctx)
        }
        "ask" => ask_ai("default"),
        "sys" | "info" => system_info(),
        "clean" => clean_project(),
        "help" | _ => quick_help(),
    }
}

pub fn git_shortcut() -> String {
    run_command("git", &["status".to_string(), "--short".to_string()])
        .unwrap_or_else(|e| e)
}

pub fn npm_shortcut() -> String {
    run_command("npm", &["--version".to_string()])
        .unwrap_or_else(|e| e)
}

pub fn docker_shortcut() -> String {
    run_command("docker", &["ps".to_string()])
        .unwrap_or_else(|e| e)
}

pub fn list_files() -> String {
    #[cfg(windows)]
    {
        run_command("dir", &["/B".to_string()]).unwrap_or_else(|e| e)
    }
    #[cfg(not(windows))]
    {
        run_command("ls", &["-la".to_string()]).unwrap_or_else(|e| e)
    }
}

pub fn init_project(ctx: &crate::project::ProjectContext) -> String {
    let name = &ctx.name;
    let template = ctx.language.as_str();

    match template {
        "javascript" | "js" | "node" => {
            let _ = run_command("mkdir", &[name.to_string()]);
            run_command("npm", &["init".to_string(), "-y".to_string()])
                .unwrap_or_else(|e| e)
        }
        "typescript" | "ts" | "rust" | "python" => {
            format!("Run: devutils new {}", name)
        }
        _ => format!("Creating project: {}", name),
    }
}

pub fn ask_ai(prompt: &str) -> String {
    if prompt.is_empty() {
        return "Usage: quick ask <question>".to_string();
    }

    use crate::ai::AIClient;
    let client = AIClient::new();
    client.generate_code(prompt).map(|(s, _)| s).unwrap_or_else(|e| e)
}

fn system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();

    format!("OS: {}\nArchitecture: {}\nHome: {}\n", os, arch, home)
}

fn clean_project() -> String {
    let mut results = vec![];

    // Node cleanup
    if std::path::Path::new("node_modules").exists() {
        results.push("Cleaning node_modules...");
    }

    // Rust cleanup
    if std::path::Path::new("target").exists() {
        results.push("Run 'cargo clean' to clean Rust build");
    }

    // Python cleanup
    if std::path::Path::new("__pycache__").exists() {
        results.push("Cleaning __pycache__...");
    }

    if results.is_empty() {
        "Nothing to clean".to_string()
    } else {
        results.join("\n")
    }
}

fn run_command(cmd: &str, args: &[String]) -> Result<String, String> {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = Command::new(cmd)
        .args(&refs)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn quick_help() -> String {
    r#"
🚀 Quick Commands (devutils q)

Git:
  q g s        - git status
  q g a        - git add -A
  q g c <msg>  - git commit
  q g p        - git push
  q g l        - git log

NPM:
  q npm i       - npm install
  q npm r <script> - npm run
  q npm t       - npm test
  q npm d       - npm run dev

Docker:
  q d ps        - docker ps
  q d up        - docker compose up
  q d down      - docker compose down
  q d logs      - docker compose logs

Project:
  q new <name>  - init project
  q clean        - clean build artifacts

AI:
  q ask <q>     - ask AI a question

System:
  q sys         - system info
  q help         - this help
"#
    .to_string()
}

pub fn quick_commands() {
    println!("{}", quick_help());
}
