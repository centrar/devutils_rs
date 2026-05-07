use std::fs;
use std::path::PathBuf;

pub fn create_project(template: &str, name: &str, target_dir: &str) -> Result<String, String> {
    let target = PathBuf::from(target_dir).join(name);
    
    if target.exists() {
        return Err(format!("Directory already exists: {:?}", target));
    }

    fs::create_dir_all(&target).map_err(|e| format!("Failed to create directory: {}", e))?;

    match template {
        "rust-cli" => create_rust_cli(&target, name)?,
        "python-cli" => create_python_cli(&target, name)?,
        "nodejs" => create_nodejs(&target, name)?,
        "go" => create_go(&target, name)?,
        "rust-lib" => create_rust_lib(&target, name)?,
        _ => return Err(format!("Unknown template: {}. Use: rust-cli, python-cli, nodejs, go, rust-lib", template)),
    }

    Ok(format!("Created '{}' project at {:?}", name, target))
}

fn create_rust_cli(target: &PathBuf, name: &str) -> Result<(), String> {
    fs::write(target.join("Cargo.toml"), format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = {{ version = "4", features = ["derive"] }}
anyhow = "1"
"#, name)).map_err(|e| e.to_string())?;

    fs::create_dir_all(target.join("src")).map_err(|e| e.to_string())?;
    fs::write(target.join("src").join("main.rs"), r#"use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "myapp")]
#[command(about = "A CLI tool")]
struct Args {}

fn main() -> Result<()> {
    println!("Hello from myapp!");
    Ok(())
}
"#).map_err(|e| e.to_string())?;

    fs::write(target.join(".gitignore"), "/target\n*.rs.bk\nCargo.lock\n").map_err(|e| e.to_string())?;
    Ok(())
}

fn create_python_cli(target: &PathBuf, name: &str) -> Result<(), String> {
    fs::write(target.join("requirements.txt"), "click>=8.0\nrich>=12.0\n").map_err(|e| e.to_string())?;
    fs::create_dir_all(target.join(name)).map_err(|e| e.to_string())?;
    fs::write(target.join(name).join("__init__.py"), "").map_err(|e| e.to_string())?;
    fs::write(target.join(name).join("main.py"), r#"import click

@click.command()
def main():
    print("Hello!")

if __name__ == '__main__':
    main()
"#).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_nodejs(target: &PathBuf, name: &str) -> Result<(), String> {
    fs::write(target.join("package.json"), format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "main": "index.js"
}}"#, name)).map_err(|e| e.to_string())?;
    fs::write(target.join("index.js"), "console.log('Hello!');\n").map_err(|e| e.to_string())?;
    Ok(())
}

fn create_go(target: &PathBuf, name: &str) -> Result<(), String> {
    fs::write(target.join("go.mod"), format!("module {}\n\ngo 1.21", name)).map_err(|e| e.to_string())?;
    fs::write(target.join("main.go"), r#"package main
import "fmt"
func main() {
    fmt.Println("Hello!")
}
"#).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_rust_lib(target: &PathBuf, name: &str) -> Result<(), String> {
    fs::write(target.join("Cargo.toml"), format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
"#, name)).map_err(|e| e.to_string())?;

    fs::create_dir_all(target.join("src")).map_err(|e| e.to_string())?;
    fs::write(target.join("src").join("lib.rs"), r#"use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
}

pub fn new_config(name: &str) -> Config {
    Config { name: name.to_string() }
}
"#).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_templates() -> String {
    "\n📦 Available templates:\n\n  rust-cli - Rust CLI with clap\n  python-cli - Python CLI with click\n  nodejs - Node.js project\n  go - Go module\n  rust-lib - Rust library\n".to_string()
}

pub fn init_project(template: &str, name: &str, target: &str) -> String {
    match create_project(template, name, target) {
        Ok(msg) => msg,
        Err(e) => format!("Error: {}", e),
    }
}