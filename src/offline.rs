//! Offline Mode - Local AI using llama.cpp

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

static LLAMA_BIN: &[&str] = &["llama.exe", "./llama.exe", "llama-cli.exe"];

pub struct OfflineAI {
    model_path: Option<PathBuf>,
    pub is_available: bool,
}

impl OfflineAI {
    pub fn new() -> Self {
        let model_path = Self::find_model();
        let is_available = model_path.is_some() && Self::find_binary().is_some();

        Self {
            model_path,
            is_available,
        }
    }

    fn find_binary() -> Option<String> {
        for bin in LLAMA_BIN {
            if Command::new(bin).arg("--version").output().is_ok() {
                return Some(bin.to_string());
            }
        }
        None
    }

    fn find_model() -> Option<PathBuf> {
        let search_paths = vec![
            PathBuf::from("models/gguf/llama2-7b-chat.Q4_K_M.gguf"),
            PathBuf::from("models/llama2-7b-chat.gguf"),
            PathBuf::from("~/.devutils/models/llama2-7b-chat.gguf"),
            PathBuf::from("C:/llama.cpp/models/llama2-7b-chat.gguf"),
        ];

        for p in search_paths {
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    pub fn chat(&self, prompt: &str) -> Result<String> {
        if !self.is_available {
            return Err(anyhow::anyhow!(
                "No local model available. Install llama.cpp and a GGUF model."
            ));
        }

        let model = self.model_path.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No model found. Place a GGUF model in the models/ folder")
        })?;

        let output = Command::new("llama.exe")
            .args([
                "-m",
                model.to_str().unwrap(),
                "-p",
                prompt,
                "-n",
                "512",
                "--no-mmap",
            ])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn install_instructions() {
        println!("\n\x1b[36m📦 Offline Mode Setup\x1b[0m\n");
        println!("1. Download llama.cpp from: https://github.com/ggerganov/llama.cpp/releases");
        println!("2. Download a GGUF model (e.g., Llama-2-7B-Chat)");
        println!("3. Place the model in: ./models/llama2-7b-chat.gguf");
        println!("4. Run: devutils chat --offline\n");
    }
}

pub fn is_online() -> bool {
    std::env::var("OPENAI_API_KEY").is_ok() || std::env::var("OPENAI_KEY").is_ok()
}
