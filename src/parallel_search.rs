//! Parallel Search - simplified version
use std::fs;
use std::path::{Path, PathBuf};
use rayon::prelude::*;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub content: String,
    pub matched_line: String,
}

pub fn parallel_search(pattern: &str, path: &str, case_sensitive: bool, max_results: usize) -> Result<Vec<SearchResult>, String> {
    let regex_pattern = if case_sensitive {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    
    let pattern_re = Regex::new(&regex_pattern)
        .map_err(|e| format!("Invalid regex: {}", e))?;
    
    let files = collect_files(Path::new(path));
    
    let results: Vec<Vec<SearchResult>> = files.par_iter()
        .filter_map(|file| {
            search_file(file, &pattern_re).ok()
        })
        .collect();
    
    let mut flat_results: Vec<SearchResult> = results.into_iter().flatten().take(max_results).collect();
    flat_results.truncate(max_results);
    
    Ok(flat_results)
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if root.is_dir() {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(collect_files(&path));
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
    } else if root.is_file() {
        files.push(root.to_path_buf());
    }
    files
}

fn search_file(file: &Path, pattern: &Regex) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results = Vec::new();
    let content = fs::read_to_string(file)?;
    
    for (line_num, line) in content.lines().enumerate() {
        if let Some(mat) = pattern.find(line) {
            results.push(SearchResult {
                file: file.to_path_buf(),
                line: line_num + 1,
                column: mat.start() + 1,
                content: line.trim().to_string(),
                matched_line: line.to_string(),
            });
        }
    }
    
    Ok(results)
}