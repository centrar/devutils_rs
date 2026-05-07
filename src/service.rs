//! Service Commands

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static SERVICES: Lazy<Mutex<HashMap<String, Service>>> =
    Lazy::new(|| Mutex::new(default_services()));

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub enabled: bool,
    pub command: String,
}

fn default_services() -> HashMap<String, Service> {
    let mut services = HashMap::new();
    services.insert(
        "github".to_string(),
        Service {
            name: "github".to_string(),
            enabled: true,
            command: "gh".to_string(),
        },
    );
    services.insert(
        "stripe".to_string(),
        Service {
            name: "stripe".to_string(),
            enabled: false,
            command: "stripe".to_string(),
        },
    );
    services
}

pub fn list_services() -> Vec<Service> {
    SERVICES.lock().unwrap().values().cloned().collect()
}

pub fn enable_service(name: &str) -> Result<String, String> {
    SERVICES
        .lock()
        .unwrap()
        .get_mut(name)
        .map(|s| s.enabled = true);
    Ok(format!("Enabled: {}", name))
}

pub fn disable_service(name: &str) -> Result<String, String> {
    SERVICES
        .lock()
        .unwrap()
        .get_mut(name)
        .map(|s| s.enabled = false);
    Ok(format!("Disabled: {}", name))
}

pub fn service_commands() {
    println!("devutils service list/enable/disable");
}
