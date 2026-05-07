use std::process::Command;
use std::path::Path;

pub struct Toolchain {
    language: String,
}

impl Toolchain {
    pub fn detect() -> Option<Self> {
        // Detect project language
        if Path::new("Cargo.toml").exists() {
            return Some(Self { language: "rust".to_string() });
        }
        if Path::new("package.json").exists() {
            return Some(Self { language: "node".to_string() });
        }
        if Path::new("requirements.txt").exists() || Path::new("setup.py").exists() {
            return Some(Self { language: "python".to_string() });
        }
        if Path::new("go.mod").exists() {
            return Some(Self { language: "go".to_string() });
        }
        if Path::new("pom.xml").exists() || Path::new("build.gradle").exists() {
            return Some(Self { language: "java".to_string() });
        }
        None
    }

    pub fn build(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["build", "--release"]),
            "node" => run_cmd("npm", &["run", "build"]),
            "python" => run_cmd("python", &["-m", "build"]),
            "go" => run_cmd("go", &["build"]),
            "java" => run_cmd("mvn", &["package"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn test(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["test"]),
            "node" => run_cmd("npm", &["test"]),
            "python" => run_cmd("pytest", &[]),
            "go" => run_cmd("go", &["test", "./..."]),
            "java" => run_cmd("mvn", &["test"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn lint(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["clippy", "--all-targets"]),
            "node" => run_cmd("npx", &["eslint", "."]),
            "python" => run_cmd("pylint", &["."]),
            "go" => run_cmd("golangci-lint", &["run"]),
            "java" => run_cmd("mvn", &["checkstyle:check"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn dev(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["run"]),
            "node" => run_cmd("npm", &["run", "dev"]),
            "python" => run_cmd("python", &["-m", "flask", "run"]),
            "go" => run_cmd("go", &["run", "."]),
            "java" => run_cmd("mvn", &["spring-boot:run"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn clean(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["clean"]),
            "node" => run_cmd("rm", &["-rf", "node_modules", "dist"]),
            "python" => run_cmd("rm", &["-rf", "__pycache__", "*.egg-info", "dist", "build"]),
            "go" => run_cmd("go", &["clean"]),
            "java" => run_cmd("mvn", &["clean"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn format(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["fmt"]),
            "node" => run_cmd("npx", &["prettier", "--write", "."]),
            "python" => run_cmd("black", &["."]),
            "go" => run_cmd("gofmt", &["-w", "."]),
            "java" => run_cmd("mvn", &["spotless:apply"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn deps(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["tree"]),
            "node" => run_cmd("npm", &["list", "--depth=0"]),
            "python" => run_cmd("pip", &["list"]),
            "go" => run_cmd("go", &["list", "-m", "all"]),
            "java" => run_cmd("mvn", &["dependency:tree"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }

    pub fn update_deps(&self) -> Result<String, String> {
        match self.language.as_str() {
            "rust" => run_cmd("cargo", &["update"]),
            "node" => run_cmd("npm", &["update"]),
            "python" => run_cmd("pip", &["list", "--outdated"]),
            "go" => run_cmd("go", &["get", "-u", "./..."]),
            "java" => run_cmd("mvn", &["versions:update-parent"]),
            _ => Err(format!("Unknown language: {}", self.language)),
        }
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    let result = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if output.status.success() {
        Ok(result)
    } else {
        Err(result)
    }
}

pub fn run_toolchain_command(command: &str) -> Result<String, String> {
    if let Some(tc) = Toolchain::detect() {
        match command {
            "build" | "b" => tc.build(),
            "test" | "t" => tc.test(),
            "lint" | "l" => tc.lint(),
            "dev" | "d" => tc.dev(),
            "clean" | "c" => tc.clean(),
            "format" | "fmt" | "f" => tc.format(),
            "deps" | "dependencies" => tc.deps(),
            "update" | "upgrade" => tc.update_deps(),
            _ => Err(format!("Unknown toolchain command: {}", command)),
        }
    } else {
        Err("No supported project found (Cargo.toml, package.json, requirements.txt, go.mod, pom.xml)".to_string())
    }
}

pub fn get_toolchain_info() -> String {
    if let Some(tc) = Toolchain::detect() {
        format!("Detected project type: {}", tc.language)
    } else {
        "No supported project found".to_string()
    }
}