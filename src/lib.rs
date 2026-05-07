//! DevUtils - The fastest AI-powered developer toolkit
//!
//! # Features
//!
//! - AI-powered code search and generation
//! - Project auto-detection
//! - High-performance file search
//! - Plugin system

// Legacy modules contain stubs and planned features not yet wired up.
// We suppress dead_code at the crate level to keep warnings focused on
// genuine issues rather than intentional scaffolding.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(deprecated)]

pub mod advanced;
pub mod error;

pub mod ai;
pub mod auth;
pub mod browser;
pub mod chat;
pub mod checkpoints;
pub mod vision;
pub mod local_models;
pub mod voice;
pub mod commands;
pub mod context;
pub mod corrections;
pub mod cron;
pub mod dev_loop;
pub mod github;
pub mod merge_resolver;
pub mod diff_view;
pub mod docker;
pub mod enterprise;
pub mod git;
pub mod git_session;
pub mod git_ui;
pub mod hooks;
pub mod incident;
pub mod integrations;
pub mod intent;
pub mod marketplace;
pub mod mcp;
pub mod multiagent;
pub mod registry;
pub mod unified_plugins;
pub mod plugin_manager;
pub mod offline;
pub mod picker;
pub mod plugin;
pub mod plugins_100;
pub mod process;
pub mod project;
pub mod providers;
pub mod quick;
pub mod recording;
pub mod search;
pub mod service;
pub mod skills;
pub mod spec;
pub mod sync;
pub mod tui;
pub mod ui;
pub mod utils;
pub mod vimmode;
pub mod viral;
pub mod vscode;
pub mod web;
pub mod worktree;
pub mod project_map;
pub mod cli_types;
pub mod shell_interpreter;
pub mod completions;
pub mod vector_store;
pub mod lsp;
pub mod sandbox;
pub mod intelligence;
pub mod ci_bridge;
pub mod model_router;
pub mod agent_state;
pub mod atomic_edit;
pub mod ultimate_agent;
pub mod all_utils;

pub use ai::{AIClient, CodeCompleter, CodeRefactorer, SearchResult, SemanticSearch};
pub use mcp::{
    add_server, get_server, is_running, list_by_category, list_servers, search_servers,
    server_count, start_server, stop_server, MCPServerConfig,
};
pub use plugin::{Plugin, PluginCommand, PluginManager};
pub use project::{Framework, ProjectContext};
pub use registry::{PluginRegistry, PluginCategory, RegistryEntry};
pub use search::{FileSearch, Grep, SearchEngine};
pub use unified_plugins::{UnifiedPluginSystem, PluginStats};
pub use vscode::{
    extension_count, get_extension, list_extensions, search_extensions, VSCodeExtension,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
