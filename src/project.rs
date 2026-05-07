//! Project Context - Auto-detect and remember project structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub framework: Framework,
    pub language: String,
    pub build_cmd: String,
    pub test_cmd: String,
    pub run_cmd: String,
    pub install_cmd: String,
    pub ports: Vec<u16>,
    pub dependencies: Vec<String>,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Framework {
    React,
    Vue,
    Svelte,
    Next,
    Nuxt,
    Astro,
    PHP,
    Elixir,
    Kotlin,
    Swift,
    CSharp,
    FastAPI,
    Flask,
    Django,
    Express,
    Nest,
    Rust,
    Go,
    Java,
    Spring,
    Laravel,
    Symfony,
    Ruby,
    Python,
    Node,
    Unknown,
}

impl ProjectContext {
    pub fn detect() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::detect_in(&cwd)
    }

    pub fn detect_in(path: &Path) -> Self {
        let mut ctx = Self {
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            framework: Framework::Unknown,
            language: "".to_string(),
            build_cmd: "".to_string(),
            test_cmd: "".to_string(),
            run_cmd: "".to_string(),
            install_cmd: "".to_string(),
            ports: vec![],
            dependencies: vec![],
            files: HashMap::new(),
        };

        let files = vec![
            (
                "package.json",
                "node",
                "npm install",
                "npm test",
                "npm run dev",
                3000,
            ),
            (
                "Cargo.toml",
                "rust",
                "cargo build",
                "cargo test",
                "cargo run",
                8080,
            ),
            ("go.mod", "go", "go build", "go test", "go run", 8080),
            (
                "requirements.txt",
                "python",
                "pip install -r requirements.txt",
                "pytest",
                "python app.py",
                8000,
            ),
            (
                "pyproject.toml",
                "python",
                "pip install -e .",
                "pytest",
                "python -m app",
                8000,
            ),
            (
                "Pipfile",
                "python",
                "pipenv install",
                "pipenv test",
                "pipenv run",
                8000,
            ),
            (
                "Gemfile",
                "ruby",
                "bundle install",
                "bundle exec rake test",
                "bundle exec ruby app.rb",
                9292,
            ),
            (
                "pom.xml",
                "java",
                "mvn package",
                "mvn test",
                "java -jar target/app.jar",
                8080,
            ),
            (
                "build.gradle",
                "java",
                "gradle build",
                "gradle test",
                "gradle run",
                8080,
            ),
            (
                "composer.json",
                "php",
                "composer install",
                "phpunit",
                "php artisan serve",
                8000,
            ),
            (
                "mix.exs",
                "elixir",
                "mix deps.get",
                "mix test",
                "mix run",
                4000,
            ),
            (
                "shard.yml",
                "crystal",
                "shards install",
                "crystal spec",
                "crystal run",
                3000,
            ),
        ];

        for (file, lang, install, test, run, port) in files {
            let file_path = path.join(file);
            if file_path.exists() {
                ctx.language = lang.to_string();
                ctx.install_cmd = install.to_string();
                ctx.test_cmd = test.to_string();
                ctx.run_cmd = run.to_string();
                ctx.build_cmd = if lang == "node" {
                    "npm run build".to_string()
                } else {
                    "".to_string()
                };
                ctx.ports.push(port);

                ctx.framework = Self::detect_framework(path, file, lang);
                ctx.name = Self::detect_name(path, file);
                break;
            }
        }

        if ctx.language.is_empty() {
            ctx.framework = Self::detect_from_files(path);
        }

        let package_json = path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                        ctx.dependencies = deps.keys().cloned().collect();
                    }
                    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                        for (name, _) in scripts {
                            ctx.files.insert(format!("script:{}", name), name.clone());
                        }
                    }
                }
            }
        }

        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                for line in content.lines() {
                    if line.starts_with("name = ") {
                        ctx.name = line
                            .trim_start_matches("name = ")
                            .trim_matches('"')
                            .to_string();
                        break;
                    }
                }
            }
        }

        ctx
    }

    fn detect_framework(path: &Path, file: &str, lang: &str) -> Framework {
        match file {
            "package.json" => {
                if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                    let json: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_default();
                    let deps = json.get("dependencies").map(|d| d.as_object()).flatten();

                    if deps.map(|d| d.contains_key("react")).unwrap_or(false) {
                        return Framework::React;
                    }
                    if deps.map(|d| d.contains_key("vue")).unwrap_or(false) {
                        return Framework::Vue;
                    }
                    if deps.map(|d| d.contains_key("svelte")).unwrap_or(false) {
                        return Framework::Svelte;
                    }
                    if deps.map(|d| d.contains_key("next")).unwrap_or(false) {
                        return Framework::Next;
                    }
                    if deps.map(|d| d.contains_key("nuxt")).unwrap_or(false) {
                        return Framework::Nuxt;
                    }
                    if deps.map(|d| d.contains_key("express")).unwrap_or(false) {
                        return Framework::Express;
                    }
                    if deps
                        .map(|d| d.contains_key("@nestjs/core"))
                        .unwrap_or(false)
                    {
                        return Framework::Nest;
                    }
                    return Framework::Node;
                }
                Framework::Node
            }
            "requirements.txt" | "pyproject.toml" | "Pipfile" => {
                let req_file = path.join("requirements.txt");
                let pyproj = path.join("pyproject.toml");

                if pyproj.exists() {
                    if let Ok(content) = fs::read_to_string(&pyproj) {
                        if content.contains("fastapi") {
                            return Framework::FastAPI;
                        }
                        if content.contains("flask") {
                            return Framework::Flask;
                        }
                        if content.contains("django") {
                            return Framework::Django;
                        }
                    }
                }

                if let Ok(content) = fs::read_to_string(&req_file) {
                    if content.contains("fastapi") {
                        return Framework::FastAPI;
                    }
                    if content.contains("flask") {
                        return Framework::Flask;
                    }
                    if content.contains("django") {
                        return Framework::Django;
                    }
                }

                if lang == "python" {
                    return Framework::Python;
                }
                Framework::Unknown
            }
            "Cargo.toml" => Framework::Rust,
            "go.mod" => Framework::Go,
            "composer.json" => Framework::PHP,
            "mix.exs" => Framework::Elixir,
            "pom.xml" | "build.gradle" => {
                let pom = path.join("pom.xml");
                if pom.exists()
                    && fs::read_to_string(&pom)
                        .map(|c| c.contains("spring"))
                        .unwrap_or(false)
                {
                    return Framework::Spring;
                }
                Framework::Java
            }
            "Gemfile" => Framework::Ruby,
            _ => Framework::Unknown,
        }
    }

    fn detect_name(path: &Path, file: &str) -> String {
        match file {
            "package.json" => {
                if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                            return name.to_string();
                        }
                    }
                }
            }
            "Cargo.toml" => {
                if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
                    for line in content.lines() {
                        if line.starts_with("name = ") {
                            return line
                                .trim_start_matches("name = ")
                                .trim_matches('"')
                                .to_string();
                        }
                    }
                }
            }
            "go.mod" => {
                if let Ok(content) = fs::read_to_string(path.join("go.mod")) {
                    for line in content.lines() {
                        if line.starts_with("module ") {
                            return line
                                .trim_start_matches("module ")
                                .split('/')
                                .last()
                                .unwrap_or("unknown")
                                .to_string();
                        }
                    }
                }
            }
            _ => {}
        }

        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn detect_from_files(path: &Path) -> Framework {
        let src_dir = path.join("src");
        let lib_dir = path.join("lib");
        let app_dir = path.join("app");

        if src_dir.exists() || lib_dir.exists() || app_dir.exists() {
            let dir = if src_dir.exists() {
                &src_dir
            } else if lib_dir.exists() {
                &lib_dir
            } else {
                &app_dir
            };
            let entries: Vec<_> = fs::read_dir(dir).into_iter().flatten().flatten().collect();

            for entry in entries {
                let path = entry.path();
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                match ext.as_str() {
                    "rs" => return Framework::Rust,
                    "go" => return Framework::Go,
                    "java" => return Framework::Java,
                    "kt" => return Framework::Kotlin,
                    "swift" => return Framework::Swift,
                    "cs" => return Framework::CSharp,
                    "rb" => return Framework::Ruby,
                    "php" => return Framework::PHP,
                    "py" => return Framework::Python,
                    "js" | "ts" | "jsx" | "tsx" => return Framework::Node,
                    _ => {}
                }
            }
        }

        if app_dir.exists() {
            let entries = fs::read_dir(&app_dir).into_iter().flatten().flatten();
            for entry in entries {
                if let Some(ext) = entry.path().extension() {
                    match ext.to_string_lossy().as_ref() {
                        "vue" => return Framework::Vue,
                        "svelte" => return Framework::Svelte,
                        "jsx" | "tsx" => return Framework::React,
                        _ => {}
                    }
                }
            }
        }

        Framework::Unknown
    }

    pub fn framework_name(&self) -> String {
        match self.framework {
            Framework::React => "React".to_string(),
            Framework::Vue => "Vue".to_string(),
            Framework::Svelte => "Svelte".to_string(),
            Framework::Next => "Next.js".to_string(),
            Framework::Nuxt => "Nuxt".to_string(),
            Framework::Astro => "Astro".to_string(),
            Framework::FastAPI => "FastAPI".to_string(),
            Framework::Flask => "Flask".to_string(),
            Framework::Django => "Django".to_string(),
            Framework::Express => "Express".to_string(),
            Framework::Nest => "NestJS".to_string(),
            Framework::Rust => "Rust".to_string(),
            Framework::Go => "Go".to_string(),
            Framework::Java => "Java".to_string(),
            Framework::Spring => "Spring".to_string(),
            Framework::Laravel => "Laravel".to_string(),
            Framework::Symfony => "Symfony".to_string(),
            Framework::Ruby => "Ruby".to_string(),
            Framework::Python => "Python".to_string(),
            Framework::Node => "Node.js".to_string(),
            Framework::PHP => "PHP".to_string(),
            Framework::Elixir => "Elixir".to_string(),
            Framework::Kotlin => "Kotlin".to_string(),
            Framework::Swift => "Swift".to_string(),
            Framework::CSharp => "C#".to_string(),
            Framework::Unknown => "Unknown".to_string(),
        }
    }

    pub fn run_test(&self) -> String {
        if !self.test_cmd.is_empty() {
            return self.test_cmd.clone();
        }

        match self.framework {
            Framework::React | Framework::Vue | Framework::Next | Framework::Nuxt => {
                "npm test".to_string()
            }
            Framework::Node => "npm test".to_string(),
            Framework::Rust => "cargo test".to_string(),
            Framework::Go => "go test ./...".to_string(),
            Framework::FastAPI | Framework::Flask | Framework::Django | Framework::Python => {
                "pytest".to_string()
            }
            Framework::Java | Framework::Spring => "mvn test".to_string(),
            Framework::Ruby => "bundle exec rspec".to_string(),
            Framework::PHP => "phpunit".to_string(),
            Framework::Elixir => "mix test".to_string(),
            _ => "echo 'No tests configured'".to_string(),
        }
    }

    pub fn run_dev(&self) -> String {
        if !self.run_cmd.is_empty() {
            return self.run_cmd.clone();
        }

        match self.framework {
            Framework::React | Framework::Vue | Framework::Next | Framework::Nuxt => {
                "npm run dev".to_string()
            }
            Framework::Svelte => "npm run dev".to_string(),
            Framework::Node => "node index.js".to_string(),
            Framework::FastAPI => "uvicorn main:app --reload".to_string(),
            Framework::Flask => "flask run".to_string(),
            Framework::Django => "python manage.py runserver".to_string(),
            Framework::Rust => "cargo run".to_string(),
            Framework::Go => "go run main.go".to_string(),
            Framework::Java => "mvn spring-boot:run".to_string(),
            Framework::Ruby => "bundle exec rails server".to_string(),
            Framework::PHP => "php artisan serve".to_string(),
            Framework::Elixir => "mix run".to_string(),
            Framework::Kotlin => "./gradlew run".to_string(),
            _ => "echo 'No run command configured'".to_string(),
        }
    }
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self::detect()
    }
}
