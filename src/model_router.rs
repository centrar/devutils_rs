//! Model Router - Intelligent model switching and fallback
//! Automatically routes requests to API (Gemini/OpenAI) or Local (Ollama) based on status.

use crate::ai::{AIClient, GenerateOptions, TokenUsage};
use crate::local_models;
use crate::error::{DevUtilsError, Result};

pub enum ModelSource {
    Cloud,
    Local,
}

pub struct ModelRouter {
    client: AIClient,
    force_local: bool,
}

impl ModelRouter {
    pub fn new(force_local: bool) -> Self {
        Self {
            client: AIClient::new(),
            force_local,
        }
    }

    pub fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_usage(prompt).map(|(s, _)| s)
    }

    pub fn generate_with_usage(&self, prompt: &str) -> Result<(String, TokenUsage)> {
        if self.force_local {
            return local_models::run_local(prompt)
                .map(|s| (s, TokenUsage::default()))
                .map_err(|e| DevUtilsError::AgentError(format!("Local model failed: {}", e)));
        }

        // Try Cloud first
        match self.client.generate_with_options(prompt, &GenerateOptions::default()) {
            Ok(res) => Ok(res),
            Err(e) => {
                eprintln!("⚠️ Cloud API failed: {}. Falling back to local model...", e);
                local_models::run_local(prompt)
                    .map(|s| (s, TokenUsage::default()))
                    .map_err(|e| DevUtilsError::AgentError(format!("Local fallback failed: {}", e)))
            }
        }
    }

    pub fn generate_code(&self, prompt: &str) -> Result<String> {
        self.generate_with_usage(prompt).map(|(s, _)| s)
    }

    /// Dynamic Orchestration - Routes sub-tasks to the best model for the job.
    /// Each phase gets a distinct temperature:
    ///   PLANNING     → 0.8  (creative, exploratory)
    ///   IMPLEMENTATION → 0.2 (precise, deterministic)
    ///   VERIFICATION → 0.0  (strictly factual)
    pub fn orchestrate(&self, task: &str, phase: &str) -> Result<(String, TokenUsage)> {
        let temperature = match phase {
            "PLANNING"        => 0.8,
            "IMPLEMENTATION"  => 0.2,
            "VERIFICATION"    => 0.0,
            _                 => 0.7, // default
        };
        let opts = GenerateOptions {
            temperature,
            ..Default::default()
        };
        self.client.generate_with_options(task, &opts)
            .map_err(|e| DevUtilsError::AgentError(format!("[{}] API failed: {}", phase, e)))
    }
}
