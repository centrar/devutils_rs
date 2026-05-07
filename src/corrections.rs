//! Rule-based corrections like thefuck

pub struct CorrectionEngine {
    rules: Vec<CorrectionRule>,
}

#[derive(Clone)]
pub struct CorrectionRule {
    pub name: String,
    pub match_fn: fn(&str) -> bool,
    pub new_command: fn(&str) -> String,
    pub enabled: bool,
}

impl CorrectionRule {
    fn matches(&self, input: &str) -> bool {
        (self.match_fn)(input)
    }

    fn apply(&self, input: &str) -> String {
        (self.new_command)(input)
    }
}

impl CorrectionEngine {
    pub fn new() -> Self {
        let rules = vec![
            CorrectionRule {
                name: "cd_parent".to_string(),
                match_fn: |cmd| cmd.contains("cd.."),
                new_command: |cmd| cmd.replace("cd..", "cd .."),
                enabled: true,
            },
            CorrectionRule {
                name: "git_branch_typo".to_string(),
                match_fn: |cmd| cmd.starts_with("git brnach") || cmd.starts_with("git brasnch"),
                new_command: |cmd| cmd.replace("brnach", "branch").replace("brasnch", "branch"),
                enabled: true,
            },
            CorrectionRule {
                name: "git_push_no_remote".to_string(),
                match_fn: |cmd| cmd.contains("git push") && !cmd.contains("git push -u"),
                new_command: |cmd| format!("{} -u origin", cmd),
                enabled: true,
            },
            CorrectionRule {
                name: "ls_dot".to_string(),
                match_fn: |cmd| cmd == "ls." || cmd == "ls .." || cmd == "ls ./",
                new_command: |_| "ls".to_string(),
                enabled: true,
            },
            CorrectionRule {
                name: "pip_install".to_string(),
                match_fn: |cmd| {
                    (cmd.starts_with("pip install") || cmd.starts_with("pip i"))
                        && !cmd.contains("requirements")
                },
                new_command: |cmd| {
                    if cmd.contains("pip i") {
                        cmd.replace("pip i", "pip install")
                    } else {
                        format!("{} -r requirements.txt", cmd)
                    }
                },
                enabled: true,
            },
            CorrectionRule {
                name: "cargo_run_release".to_string(),
                match_fn: |cmd| cmd.contains("cargo run") && !cmd.contains("--release"),
                new_command: |cmd| format!("{} --release", cmd),
                enabled: true,
            },
        ];

        Self { rules }
    }

    pub fn correct(&self, input: &str) -> Option<String> {
        for rule in &self.rules {
            if rule.enabled && rule.matches(input) {
                return Some(rule.apply(input));
            }
        }
        None
    }

    pub fn suggest(&self, input: &str) -> Vec<(String, String)> {
        let mut suggestions = Vec::new();

        if let Some(corrected) = self.correct(input) {
            suggestions.push((corrected.clone(), self.find_rule_name(&corrected)));
            return suggestions;
        }

        let lower = input.to_lowercase();

        if lower.contains("permission denied") && std::env::consts::OS != "windows" {
            suggestions.push((format!("sudo {}", input), "run with sudo".to_string()));
        }

        if lower.contains("not found") || lower.contains("command not found") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if let Some(cmd) = parts.first() {
                suggestions.push((
                    format!("apt install {}", cmd),
                    "install via apt".to_string(),
                ));
                suggestions.push((
                    format!("brew install {}", cmd),
                    "install via homebrew".to_string(),
                ));
            }
        }

        if lower.contains("no such file") || lower.contains("not exist") {
            suggestions.push((
                input
                    .replace("import", "pip install")
                    .replace("npm", "npm install"),
                "install dependency".to_string(),
            ));
        }

        suggestions
    }

    pub fn find_rule_name(&self, corrected: &str) -> String {
        for rule in &self.rules {
            if rule.matches(corrected) {
                return rule.name.to_string();
            }
        }
        "auto-corrected".to_string()
    }

    pub fn explain(&self, input: &str) -> String {
        if let Some(corrected) = self.correct(input) {
            return format!(
                "Corrected: '{}' to '{}'\nRule: {}",
                input,
                corrected,
                self.find_rule_name(&corrected)
            );
        }

        let mut explanation = Vec::new();

        if input.contains("cd..") {
            explanation.push("Did you mean: cd .. (space between cd and ..)");
        }
        if input.starts_with("git ") && input.contains("brnach") {
            explanation.push("Did you mean: git branch");
        }
        if input.contains("pip i") {
            explanation.push("Did you mean: pip install -r requirements.txt");
        }
        if input.starts_with("cargo ") && !input.contains("cargo install") {
            explanation.push("Did you mean: cargo run --release");
        }

        if explanation.is_empty() {
            if let Some(suggestion) = self.suggest(input).first() {
                return format!("Try: {}", suggestion.0);
            }
            format!("No correction found for: {}", input)
        } else {
            explanation.join("\n")
        }
    }
}

impl Default for CorrectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn did_you_mean(input: &str) -> String {
    let engine = CorrectionEngine::new();

    if let Some(corrected) = engine.correct(input) {
        return corrected;
    }

    if let Some((cmd, _)) = engine.suggest(input).first() {
        return cmd.to_string();
    }

    input.to_string()
}

pub fn fix_command(input: &str) -> String {
    did_you_mean(input)
}
