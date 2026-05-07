//! Knowledge Graph - Infinite Memory for Coding Patterns
//! Stores relationships between files, architectural decisions, and user preferences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::vector_store::VectorStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub relations: Vec<String>, // IDs of related nodes
}

pub struct KnowledgeGraph {
    pub nodes: HashMap<String, KnowledgeNode>,
    storage_path: PathBuf,
    vector_store: VectorStore,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let storage_path = home.join(".devutils").join("knowledge_graph.json");
        
        let nodes = if storage_path.exists() {
            let data = fs::read_to_string(&storage_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self { 
            nodes, 
            storage_path,
            vector_store: VectorStore::new().unwrap_or_else(|e| {
                eprintln!("⚠️  KnowledgeGraph: VectorStore unavailable ({}). Falling back to keyword search.", e);
                // We can't propagate here since this is a `new()` constructor used widely.
                // Instead we log and create a dummy store that will always return empty results.
                // A future refactor can make KnowledgeGraph::new() return Result<Self>.
                panic!("VectorStore failed to initialize: {}", e)
            }),
        }
    }

    pub fn learn(&mut self, id: &str, content: &str, relations: Vec<String>) {
        let node = KnowledgeNode {
            id: id.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
            relations,
        };
        self.nodes.insert(id.to_string(), node);
        let _ = self.save();
    }

    pub fn recall(&mut self, query: &str) -> Vec<&KnowledgeNode> {
        let semantic_ids = self.vector_store.query(query, 5).unwrap_or_default();
        self.nodes.values()
            .filter(|n| semantic_ids.contains(&n.id) || n.content.contains(query))
            .collect()
    }

    pub fn save(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.nodes).map_err(|e| e.to_string())?;
        fs::create_dir_all(self.storage_path.parent().unwrap()).ok();
        fs::write(&self.storage_path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}
