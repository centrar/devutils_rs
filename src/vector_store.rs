//! Vector Store - Real RAG implementation using fastembed
//! Uses local miniLM for true semantic vector embeddings.

use std::collections::HashMap;
use crate::project_map::ProjectMap;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use crate::error::{DevUtilsError, Result};

pub struct VectorStore {
    index: HashMap<String, (String, Vec<f32>)>, // Path -> (Content, Embedding)
    model: TextEmbedding,
}

impl VectorStore {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(false))
            .map_err(|e| DevUtilsError::VectorStoreError(format!("Failed to init fastembed: {}", e)))?;
        
        Ok(Self { index: HashMap::new(), model })
    }

    pub fn index_project(&mut self, path: &str) -> Result<()> {
        let map = ProjectMap::new(path);
        let summary = map.generate_summary();
        
        let mut paths = Vec::new();
        let mut blocks = Vec::new();
        
        for block in summary.split("\n\n") {
            if let Some(first_line) = block.lines().next() {
                if first_line.contains("---") {
                    let file_path = first_line.trim_matches(|c| c == '-' || c == ' ').to_string();
                    paths.push(file_path);
                    blocks.push(block.to_string());
                }
            }
        }
        
        if blocks.is_empty() { return Ok(()); }
        
        let embeddings = self.model.embed(blocks.clone(), None)
            .map_err(|e| DevUtilsError::VectorStoreError(format!("Embedding failed: {}", e)))?;
            
        for (i, embedding) in embeddings.into_iter().enumerate() {
            self.index.insert(paths[i].clone(), (blocks[i].clone(), embedding));
        }
        Ok(())
    }

    pub fn query(&mut self, text: &str, limit: usize) -> Result<Vec<String>> {
        let query_embedding = self.model.embed(vec![text.to_string()], None)
            .map_err(|e| DevUtilsError::VectorStoreError(format!("Query embedding failed: {}", e)))?
            .pop().unwrap();
            
        let mut results: Vec<(f32, String)> = self.index.iter()
            .map(|(_, (content, emb))| {
                let score = cosine_similarity(&query_embedding, emb);
                (score, content.clone())
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(results.into_iter().take(limit).map(|(_, c)| c).collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}
