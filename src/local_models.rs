//! Local Models - Support for Ollama, LM Studio, llama.cpp

use std::collections::HashMap;
use std::process::Command;

pub fn check_ollama() -> bool {
    Command::new("ollama")
        .arg("list")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_lmstudio() -> bool {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return std::path::Path::new(&appdata).join("LM Studio").exists();
    }
    false
}

pub fn check_providers() -> HashMap<String, bool> {
    let mut providers = HashMap::new();
    providers.insert("ollama".to_string(), check_ollama());
    providers.insert("lm_studio".to_string(), check_lmstudio());
    providers
}

pub fn list_ollama_models() -> Vec<String> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .ok();
    
    if let Some(o) = output {
        if o.status.success() {
            return String::from_utf8_lossy(&o.stdout)
                .lines()
                .skip(1)
                .filter_map(|line| {
                    let name = line.split_whitespace().next()?;
                    if name.is_empty() { None } else { Some(name.to_string()) }
                })
                .collect();
        }
    }
    vec![]
}

pub fn run_ollama(model: &str, prompt: &str) -> Result<String, String> {
    let output = Command::new("ollama")
        .arg("run")
        .arg(model)
        .arg(prompt)
        .output()
        .map_err(|e| format!("Failed: {}", e))?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn list_lmstudio_models() -> Vec<String> {
    let client = reqwest::blocking::Client::new();
    if let Ok(resp) = client.get("http://localhost:1234/v1/models").send() {
        if let Ok(json) = resp.json::<serde_json::Value>() {
            return json.get("data")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
        }
    }
    vec![]
}

pub fn run_lmstudio(model: &str, messages: &[(String, String)]) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    
    let msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|(role, content)| {
            serde_json::json!({"role": role, "content": content})
        })
        .collect();
    
    let payload = serde_json::json!({
        "model": model,
        "messages": msgs,
        "temperature": 0.7,
    });
    
    let body = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    
    let resp = client.post("http://localhost:1234/v1/chat/completions")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| format!("Parse error: {}", e))?;
    
    json.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|co| co.as_str())
        .map(String::from)
        .ok_or_else(|| "No response".to_string())
}

pub fn run_local(prompt: &str) -> Result<String, String> {
    if check_ollama() {
        let models = list_ollama_models();
        if let Some(model) = models.first() {
            return run_ollama(model, prompt);
        }
    }
    
    if check_lmstudio() {
        let models = list_lmstudio_models();
        if let Some(model) = models.first() {
            return run_lmstudio(model, &[("user".to_string(), prompt.to_string())]);
        }
    }
    
    Err("No local provider available. Install Ollama (ollama.ai) or LM Studio.".to_string())
}

pub fn install_ollama() -> Result<(), String> {
    if check_ollama() {
        return Ok(());
    }
    
    println!("Installing Ollama...");
    
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-Command", "irm https://ollama.ai/install.ps1 | iex"])
            .output()
            .map_err(|e| format!("Install failed: {}", e))?;
        
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }
    
    Ok(())
}

pub fn get_status() -> String {
    let providers = check_providers();
    let mut status = String::from("Local Providers:\n");
    
    for (name, available) in &providers {
        status.push_str(&format!(
            "  {}: {}\n",
            name,
            if *available { "available" } else { "not found" }
        ));
    }
    
    if providers.values().any(|v| *v) {
        status.push_str("\nAvailable models:\n");
        
        if providers.get("ollama").copied().unwrap_or(false) {
            for m in list_ollama_models() {
                status.push_str(&format!("  - {}\n", m));
            }
        }
        
        if providers.get("lm_studio").copied().unwrap_or(false) {
            for m in list_lmstudio_models() {
                status.push_str(&format!("  - {} (LM Studio)\n", m));
            }
        }
    }
    
    status
}