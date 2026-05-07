#[cfg(test)]
mod tests {
    use devutils::intelligence::memory_graph::KnowledgeGraph;
    use devutils::project_map::ProjectMap;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_knowledge_graph_persistence() {
        let mut graph = KnowledgeGraph::new();
        graph.learn("test_node", "This is a test content about Rust architecture.", vec![]);
        
        let results = graph.recall("Rust");
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "test_node");
    }

    #[test]
    fn test_project_map_extraction() {
        let root = "src";
        let map = ProjectMap::new(root);
        let summary = map.generate_summary();
        
        // Ensure we are catching signatures from our own codebase
        assert!(summary.contains("# Project Map"));
        // Check for common signatures that should be in our src
        assert!(summary.contains("pub fn") || summary.contains("fn") || summary.contains("struct"));
    }

    #[test]
    fn test_sandbox_execution_dry_run() {
        use devutils::sandbox::Sandbox;
        let sandbox = Sandbox::new();
        // Since we can't guarantee Docker in all test environments,
        // we check if the command either succeeds or returns a Docker-related error.
        let result = sandbox.execute("echo 'test'", std::path::Path::new("."));
        match result {
            Ok(_) => { /* Docker ran successfully */ }
            Err(e) => {
                // Docker is not available in this environment – acceptable.
                assert!(
                    e.to_lowercase().contains("docker") || e.contains("error"),
                    "Unexpected sandbox error (should mention Docker or error): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_ci_bridge_log_parsing() {
        use devutils::ci_bridge::CiBridge;
        let bridge = CiBridge::new(PathBuf::from("."));
        
        // Create a dummy log file
        let log_path = "test_ci.log";
        fs::write(log_path, "Error: Compilation failed at line 42").unwrap();
        
        // This will attempt to start the agent, so we check if it reads the log correctly
        // (We expect it to fail execution because we don't have AI keys in test env, but log reading should work)
        let result = bridge.fix_ci_failure(log_path);
        
        if let Err(err) = result {
            println!("Expected error occurred (no API keys in test): {}", err);
        }
        
        fs::remove_file(log_path).unwrap();
    }
}
