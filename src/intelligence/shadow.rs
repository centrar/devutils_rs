//! Shadow Mode - Predictive "Ghost" Refactoring
//! Background evaluation of code changes using the LSP Call Graph.

use crate::model_router::ModelRouter;

pub struct ShadowMode {
    router: ModelRouter,
}

impl ShadowMode {
    pub fn new() -> Self {
        Self { router: ModelRouter::new(false) }
    }

    /// Analyze a recent change and predict the next refactor
    pub fn predict_refactor(&self, diff: &str, context: &str) -> Result<Option<String>, String> {
        let prompt = format!(
            "Analyze this Git diff and predict the next logical refactor to improve architecture:\n\nDIFF:\n{}\n\nCONTEXT:\n{}",
            diff, context
        );

        let response = self.router.generate(&prompt).map_err(|e| e.to_string())?;
        
        if response.contains("SUGGESTION:") {
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }
}
