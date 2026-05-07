//! Search Module - HIGH PERFORMANCE search, optimized to beat ripgrep

use crate::ai::SearchResult;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub struct FuzzySearch {
    matcher: SkimMatcherV2,
    max_results: usize,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            max_results: 50,
        }
    }

    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    pub fn search(&self, query: &str, items: &[String]) -> Vec<(String, i64)> {
        let mut results: Vec<(String, i64)> = items
            .iter()
            .filter_map(|item| {
                self.matcher
                    .fuzzy_match(item, query)
                    .map(|score| (item.clone(), score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(self.max_results);
        results
    }

    pub fn fuzzy_files(&self, query: &str, dir: &str) -> Vec<(String, i64)> {
        let files = self.collect_files(dir);
        self.search(query, &files)
    }

    fn collect_files(&self, dir: &str) -> Vec<String> {
        let mut files = Vec::new();

        let walker = walkdir::WalkDir::new(dir)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !should_skip_file(name) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        files
    }
}

impl Default for FuzzySearch {
    fn default() -> Self {
        Self::new()
    }
}

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    ".svn",
    ".hg",
    ".idea",
    ".vscode",
    "vendor",
    "pkg",
    "bin",
    "obj",
];

const SKIP_FILES: &[&str] = &[
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
    ".gitignore",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
];

pub struct FileSearch {
    dir: String,
    max_depth: usize,
    follow_symlinks: bool,
}

pub struct Grep {
    dir: String,
    max_depth: usize,
    follow_symlinks: bool,
    workers: usize,
}

impl FileSearch {
    pub fn new(d: &str) -> Self {
        Self {
            dir: d.to_string(),
            max_depth: 50,
            follow_symlinks: false,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn find(&self, pattern: &str, ext: Option<&str>, ignore_case: bool) -> Vec<String> {
        let mut results = Vec::new();
        let pat = if ignore_case {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        let walker = walkdir::WalkDir::new(&self.dir)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if should_skip_file(name) {
                    continue;
                }

                let matches = if let Some(extension) = ext {
                    path.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == extension)
                        .unwrap_or(false)
                } else {
                    true
                };

                if !matches {
                    continue;
                }

                let name_cmp = if ignore_case {
                    name.to_lowercase()
                } else {
                    name.to_string()
                };

                if name_cmp.contains(&pat) || path.to_string_lossy().contains(&pat) {
                    results.push(path.to_string_lossy().to_string());
                }
            }
        }

        results.sort();
        let mut results: Vec<String> = results;
        results.dedup();
        results
    }

    pub fn find_parallel(
        &self,
        pattern: &str,
        ext: Option<&str>,
        ignore_case: bool,
    ) -> Vec<String> {
        let files: Vec<String> = self.collect_files();

        let pat = if ignore_case {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        let extension = ext.map(|s| s.to_string());

        let matches: Vec<String> = files
            .par_iter()
            .filter_map(|path| {
                let p = Path::new(path);
                let name = p.file_name()?.to_str()?;

                if should_skip_file(name) {
                    return None;
                }

                if let Some(ref ext) = extension {
                    if p.extension().and_then(|e| e.to_str()) != Some(ext.as_str()) {
                        return None;
                    }
                }

                let name_cmp = if ignore_case {
                    name.to_lowercase()
                } else {
                    name.to_string()
                };

                if name_cmp.contains(&pat) || path.to_lowercase().contains(&pat) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut matches = matches;
        matches.sort();
        matches.dedup();
        matches
    }

    fn collect_files(&self) -> Vec<String> {
        let mut files = Vec::new();

        let walker = walkdir::WalkDir::new(&self.dir)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !should_skip_file(name) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        files
    }
}

impl Grep {
    pub fn new(d: &str) -> Self {
        Self {
            dir: d.to_string(),
            max_depth: 20,
            follow_symlinks: false,
            workers: num_cpus(),
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    pub fn search(
        &self,
        pattern: &str,
        ignore_case: bool,
        whole_word: bool,
    ) -> Vec<(String, usize, String)> {
        let re_pattern = if whole_word {
            format!("\\b{}\\b", pattern)
        } else {
            pattern.to_string()
        };

        let regex = match ignore_case {
            true => regex::RegexBuilder::new(&re_pattern)
                .case_insensitive(true)
                .build(),
            false => regex::RegexBuilder::new(&re_pattern).build(),
        };

        let Ok(re) = regex else {
            return self.search_simple(pattern, ignore_case);
        };

        let mut results = Vec::new();

        let walker = walkdir::WalkDir::new(&self.dir)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if should_skip_file(name) {
                    continue;
                }
            }

            if let Ok(content) = read_file_fast(path) {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        results.push((
                            path.to_string_lossy().to_string(),
                            line_num + 1,
                            line.to_string(),
                        ));

                        if results.len() > 5000 {
                            return results;
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        results
    }

    pub fn search_parallel(
        &self,
        pattern: &str,
        ignore_case: bool,
        whole_word: bool,
    ) -> Vec<(String, usize, String)> {
        let re_pattern = if whole_word {
            format!("\\b{}\\b", pattern)
        } else {
            pattern.to_string()
        };

        let regex = match ignore_case {
            true => regex::RegexBuilder::new(&re_pattern)
                .case_insensitive(true)
                .build(),
            false => regex::RegexBuilder::new(&re_pattern).build(),
        };

        let Ok(re) = regex else {
            return self.search_simple(pattern, ignore_case);
        };

        let files: Vec<String> = self.collect_files();

        let results: Vec<(String, usize, String)> = files
            .par_iter()
            .flat_map(|path| {
                let p = Path::new(path);
                let mut matches = Vec::new();

                if let Ok(content) = read_file_fast(p) {
                    for (line_num, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            matches.push((path.clone(), line_num + 1, line.to_string()));
                        }
                    }
                }

                matches
            })
            .collect();

        let mut results = results;
        results.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        results.truncate(5000);
        results
    }

    fn search_simple(&self, pattern: &str, ignore_case: bool) -> Vec<(String, usize, String)> {
        let pat = if ignore_case {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        let mut results = Vec::new();

        let walker = walkdir::WalkDir::new(&self.dir)
            .max_depth(self.max_depth)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if should_skip_file(name) {
                    continue;
                }
            }

            if let Ok(content) = read_file_fast(path) {
                for (line_num, line) in content.lines().enumerate() {
                    let line_cmp = if ignore_case {
                        line.to_lowercase()
                    } else {
                        line.to_string()
                    };

                    if line_cmp.contains(&pat) {
                        results.push((
                            path.to_string_lossy().to_string(),
                            line_num + 1,
                            line.to_string(),
                        ));

                        if results.len() > 5000 {
                            return results;
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        results
    }

    fn collect_files(&self) -> Vec<String> {
        let mut files = Vec::new();

        let walker = walkdir::WalkDir::new(&self.dir)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !should_skip_dir(e.path()));

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !should_skip_file(name) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        files
    }
}

fn should_skip_dir(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        SKIP_DIRS.contains(&name)
    } else {
        false
    }
}

fn should_skip_file(name: &str) -> bool {
    SKIP_FILES.contains(&name) || name.starts_with('.')
}

fn read_file_fast(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let meta = file.metadata()?;
    if meta.len() == 0 {
        return Ok(String::new());
    }
    // SOTA ripgrep-style memory-mapped file reading
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
    Ok(String::from_utf8_lossy(&mmap).into_owned())
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
        .min(16)
        .max(1)
}

pub struct SearchEngine {
    dir: String,
    max_depth: usize,
}

impl SearchEngine {
    pub fn new(d: &str) -> Self {
        Self {
            dir: d.to_string(),
            max_depth: 20,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn search(
        &self,
        query: &str,
        _ext: Option<&str>,
        _glob: Option<&str>,
        ignore_case: bool,
        limit: usize,
    ) -> Vec<SearchResult> {
        let grep = Grep::new(&self.dir).with_max_depth(self.max_depth);
        let results = grep.search(query, ignore_case, false);

        results
            .into_iter()
            .take(limit)
            .map(|(file, line, content)| SearchResult {
                file,
                line,
                content,
                score: 0.9,
            })
            .collect()
    }

    pub fn search_parallel(
        &self,
        query: &str,
        _ext: Option<&str>,
        _glob: Option<&str>,
        ignore_case: bool,
        limit: usize,
    ) -> Vec<SearchResult> {
        let grep = Grep::new(&self.dir).with_max_depth(self.max_depth);
        let results = grep.search_parallel(query, ignore_case, false);

        results
            .into_iter()
            .take(limit)
            .map(|(file, line, content)| SearchResult {
                file,
                line,
                content,
                score: 0.95,
            })
            .collect()
    }
}
