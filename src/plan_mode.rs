//! Plan-First Mode - Review changes before applying

pub fn run_plan_mode(task: &str) -> String {
    let client = crate::ai::AIClient::new();
    
    client.generate_code(&format!(
        "Create a step-by-step plan for: {}\n\n\
         Format as:\n\
         1. FILE: path/to/file\n\
            CHANGE: description\n\
         2. ...\n\n\
         Ask for approval before I execute.",
        task
    )).unwrap_or_else(|e| e)
}

pub fn show_plan_review(changes: &[(String, String, String)]) -> String {
    let mut output = String::from("📋 PROPOSED CHANGES\n====================\n\n");
    
    for (i, (path, action, desc)) in changes.iter().enumerate() {
        let icon = match action.as_str() {
            "create" => "📄",
            "modify" => "✏️",
            "delete" => "🗑️",
            _ => "📝",
        };
        output.push_str(&format!("{}. {} {}\n   {}: {}\n\n", i+1, icon, path, action, desc));
    }
    
    output.push_str("====================\nType 'approve' to apply, 'reject' to cancel");
    output
}