//! Intent Engine - High-level intent to implementation

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

static INTENTS: Lazy<Mutex<HashMap<String, IntentSpec>>> =
    Lazy::new(|| Mutex::new(default_intents()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSpec {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentStep {
    pub name: String,
    pub prompt: String,
}

fn default_intents() -> HashMap<String, IntentSpec> {
    let mut intents = HashMap::new();
    intents.insert(
        "build-rest-api".to_string(),
        IntentSpec {
            name: "build-rest-api".to_string(),
            description: "Build REST API".to_string(),
        },
    );
    intents.insert(
        "fix-bug".to_string(),
        IntentSpec {
            name: "fix-bug".to_string(),
            description: "Fix a bug".to_string(),
        },
    );
    intents.insert(
        "add-feature".to_string(),
        IntentSpec {
            name: "add-feature".to_string(),
            description: "Add feature".to_string(),
        },
    );
    intents
}

pub fn list_intents() -> Vec<IntentSpec> {
    let intents = INTENTS.lock().unwrap();
    intents.values().cloned().collect()
}

pub fn execute_intent(name: &str, target: &str) -> Result<String, String> {
    let intents = INTENTS.lock().unwrap();

    let spec = intents
        .get(name)
        .ok_or_else(|| format!("Intent not found: {}", name))?;

    let ai = crate::ai::AIClient::new();
    let prompt = format!("{} for: {}", spec.description, target);
    let (result, _) = ai.generate(&prompt).unwrap_or_else(|e| (e, crate::ai::TokenUsage::default()));
    Ok(result)
}

pub fn create_custom_intent(name: &str, description: &str) -> Result<String, String> {
    let mut intents = INTENTS.lock().unwrap();
    intents.insert(
        name.to_string(),
        IntentSpec {
            name: name.to_string(),
            description: description.to_string(),
        },
    );
    Ok(format!("Created: {}", name))
}

pub fn intent_commands() {
    println!("devutils intent list/run/create");
}
