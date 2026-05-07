//! Project Mapping Module - Aider-style structural awareness
//! Extracts signatures (classes, functions, types) to provide LLM context without token bloat.

use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct ProjectMap {
    pub root: String,
}

impl ProjectMap {
    pub fn new(root: &str) -> Self {
        Self { root: root.to_string() }
    }

    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("# Project Map\n\n");

        let walker = walkdir::WalkDir::new(&self.root)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| !is_ignored(e.path()));

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                if let Some(sigs) = self.extract_signatures(entry.path()) {
                    summary.push_str(&format!("## {}\n```\n{}```\n\n", 
                        entry.path().display(), sigs));
                }
            }
        }

        summary
    }

    fn extract_signatures(&self, path: &Path) -> Option<String> {
        let extension = path.extension()?.to_str()?;
        let content = std::fs::read_to_string(path).ok()?;
        
        match extension {
            "rs" => self.parse_rust(&content),
            "js" | "ts" => self.parse_javascript(&content),
            "py" => self.parse_python(&content),
            _ => None,
        }
    }

    fn parse_rust(&self, content: &str) -> Option<String> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).ok()?;
        let tree = parser.parse(content, None)?;
        
        let query_str = "(function_item name: (identifier) @name) @func
                        (struct_item name: (type_identifier) @name) @struct
                        (impl_item type: (type_identifier) @name) @impl
                        (mod_item name: (identifier) @name) @mod
                        (const_item name: (identifier) @name) @const
                        (use_declaration argument: (_) @import)";
        
        self.run_query(tree_sitter_rust::language(), &tree, content, query_str)
    }

    fn parse_javascript(&self, content: &str) -> Option<String> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_javascript::language()).ok()?;
        let tree = parser.parse(content, None)?;
        
        let query_str = "(function_declaration name: (identifier) @name) @func
                        (class_declaration name: (identifier) @name) @class
                        (method_definition name: (property_identifier) @name) @method
                        (import_statement (import_clause (identifier) @import))";
        
        self.run_query(tree_sitter_javascript::language(), &tree, content, query_str)
    }

    fn parse_python(&self, content: &str) -> Option<String> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_python::language()).ok()?;
        let tree = parser.parse(content, None)?;
        
        let query_str = "(function_definition name: (identifier) @name) @func
                        (class_definition name: (identifier) @name) @class";
        
        self.run_query(tree_sitter_python::language(), &tree, content, query_str)
    }

    fn run_query(&self, lang: tree_sitter::Language, tree: &tree_sitter::Tree, content: &str, query_str: &str) -> Option<String> {
        let query = Query::new(lang, query_str).ok()?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
        
        let mut result = String::new();
        let mut last_end = 0;
        
        for m in matches {
            for capture in m.captures {
                // To prevent duplicates from multiple captures in the same match
                if capture.node.start_byte() < last_end { continue; }
                
                let text = &content[capture.node.start_byte()..capture.node.end_byte()];
                // For functions, structs, impls, we want the signature/declaration without the full body.
                // We split by '{' and take the first part, which is usually the signature.
                if let Some(signature) = text.split('{').next() {
                    let clean_sig = signature.trim().replace('\n', " ");
                    if !clean_sig.is_empty() {
                        result.push_str("  - ");
                        result.push_str(&clean_sig);
                        result.push_str(" { ... }\n");
                        last_end = capture.node.end_byte();
                    }
                }
            }
        }
        
        if result.is_empty() { None } else { Some(result) }
    }
}

fn is_ignored(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name == "target" || name == "node_modules" || name == ".git" || name == "dist"
}
