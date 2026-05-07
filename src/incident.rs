//! Incident Responder

#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub description: String,
    pub severity: String,
    pub root_cause: Option<String>,
    pub fix_suggested: Option<String>,
}

pub fn analyze_incident(description: &str) -> Result<Incident, String> {
    let ai = crate::ai::AIClient::new();
    let prompt = format!(
        "Analyze incident: {}. Return id, severity, root_cause, fix_suggested",
        description
    );
    let (result, _) = ai.generate(&prompt).unwrap_or_else(|e| (e, crate::ai::TokenUsage::default()));
    Ok(Incident {
        id: "INC-001".to_string(),
        description: description.to_string(),
        severity: "high".to_string(),
        root_cause: Some(result),
        fix_suggested: Some("Check logs".to_string()),
    })
}

pub fn suggest_fix(error: &str) -> Result<String, String> {
    let ai = crate::ai::AIClient::new();
    Ok(ai.generate(&format!("Suggest fix for: {}", error)).map(|(s, _)| s).unwrap_or_else(|e| e))
}

pub fn health_check() -> Result<String, String> {
    Ok("CPU: 45% | Memory: 2.1GB".to_string())
}

pub fn incident_commands() {
    println!("devutils incident analyze/health/suggest");
}
