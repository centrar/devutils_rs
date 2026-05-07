//! CLI Types - Shared definitions for the binary and library
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "devutils")]
#[command(about = "🚀 The fastest AI-powered developer toolkit", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true, hide = true)]
    pub no_color: bool,

    #[arg(long, global = true)]
    pub completions: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search the codebase using semantic or keyword search
    Search {
        /// Pattern to search for
        pattern: String,
        /// Search semantically (AI-powered)
        #[arg(long, short)]
        semantic: bool,
        /// Use parallel search (faster)
        #[arg(long, short)]
        parallel: bool,
        /// Case sensitive
        #[arg(long, short)]
        case_sensitive: bool,
        /// Max results
        #[arg(long, short, default_value = "100")]
        max_results: usize,
    },
    /// Find files by name pattern
    Find { pattern: String },
    /// Grep search through files
    Grep { pattern: String },
    /// Generate code from AI
    Generate { prompt: Vec<String> },
    /// Explain code using AI
    Explain { code: Option<String> },
    /// Debug code
    Debug { code: Vec<String> },
    /// Generate tests
    Tests { code: Vec<String> },
    /// Show version
    Version,
    /// Show system info
    System,
    /// Project info
    Project,
    /// Interactive TUI
    Interactive,
    /// Launch TUI
    Tui { subcommand: Option<String> },
    /// Interactive REPL mode
    Repl,
    /// AI commands
    Ai {
        subcommand: Option<String>,
        args: Vec<String>,
        #[arg(long, short)]
        grounding: bool,
        #[arg(long, short)]
        file: Option<String>,
        #[arg(long, short)]
        system: Option<String>,
        #[arg(long, short)]
        cache: Option<String>,
    },
    /// Plugin management
    Plugin {
        subcommand: String,
        name: Option<String>,
        args: Vec<String>,
    },
    /// Browse and install plugins from marketplace
    Marketplace {
        /// Subcommand: search, install, featured, categories
        subcommand: String,
        /// Plugin name to search/install
        query: Option<String>,
    },
    /// Git status
    Status,
    /// List commits
    Commits,
    /// List branches
    Branches,
    /// Git commit
    Commit { message: String },
    /// Git push
    Push,
    /// Generate .gitignore
    Ignore,
    /// Browser commands
    Browser {
        subcommand: String,
        url: Option<String>,
    },
    /// Show local IP
    LocalIp,
    /// Check port
    Port { port: u16 },
    /// Run dev server
    Run { command: Option<String> },
    /// Run tests
    Test,
    /// Build project
    Build,
    /// Fuzzy picker for files/branches/commits
    Pick {
        /// Source: file, branch, commit, process
        source: Option<String>,
    },
    /// Cloud sync (push/pull/configure)
    Sync {
        /// Subcommand: push, pull, configure, status, list
        subcommand: String,
        /// Backend: file, http
        backend: Option<String>,
        /// Path for sync location
        path: Option<String>,
    },
    /// Hooks - automation hooks
    Hooks {
        /// Subcommand: list, get, enable, disable, run
        subcommand: String,
    },
    /// Skills - reusable skill prompts
    Skills {
        /// Subcommand: list, get, add, remove, search
        subcommand: String,
        /// Skill name
        name: Option<String>,
    },
    /// MCP - Model Context Protocol servers
    Mcp {
        /// Subcommand: list, start, stop, install, remove
        subcommand: String,
        /// Server name
        server: Option<String>,
    },
    /// Docker commands
    Docker {
        /// Subcommand: ps, logs, exec, stop, remove
        subcommand: String,
        /// Container name/id
        container: Option<String>,
    },
    /// Cron - scheduled jobs
    Cron {
        /// Subcommand: list, add, remove, run
        subcommand: String,
    },
    /// Checkpoints - save/restore state
    Checkpoints {
        /// Subcommand: list, create, restore, delete
        subcommand: String,
        /// Checkpoint name
        name: Option<String>,
    },
    /// Autonomous AI Agent v2 (Industrial Grade)
    Agent {
        /// The task for the agent to complete
        task: Vec<String>,
        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell (bash, zsh, fish, powershell)
        shell: String,
    },
    /// Intents - automated workflows
    Intents {
        /// Subcommand: list, run, create
        subcommand: String,
        /// Intent name
        name: Option<String>,
    },
    /// VS Code extensions
    Vscode {
        /// Subcommand: search, install, list, uninstall
        subcommand: String,
        /// Extension name
        extension: Option<String>,
    },
    /// Git worktrees
    Worktree {
        /// Subcommand: list, create, remove
        subcommand: String,
        /// Worktree name
        name: Option<String>,
    },
    /// Autonomous AI agent - execute tasks automatically
    Autonomous {
        /// Task description
        task: String,
        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
    /// Multi-agent orchestration
    Multiagent {
        /// Subcommand: spawn, list, stats
        subcommand: String,
        /// Agent name (for spawn)
        name: Option<String>,
        /// Task description (for spawn)
        task: Option<String>,
        /// Agent ID (for stats)
        id: Option<String>,
    },
    /// Offline AI (local models)
    Offline {
        /// Subcommand: chat, install, status
        subcommand: String,
        /// Prompt
        prompt: Option<String>,
    },
    /// All-in-One Developer Utilities
    Utils {
        /// Utility name: uuid, now, pass, jsonfmt, hex, rev, gitlog, etc.
        name: String,
        /// Args for the utility
        args: Vec<String>,
    },
    /// Run utility pipeline
    Pipe {
        /// Pipeline string (e.g., "du . | sort")
        pipeline: String,
    },
    /// Services - running services
    Services {
        /// Subcommand: list, enable, disable
        subcommand: String,
    },
    /// Quick commands
    Quick {
        /// Command to run
        command: String,
    },
    /// Recording - terminal sessions
    Recording {
        /// Subcommand: start, stop, list, play
        subcommand: String,
    },
    /// OpenAPI specs
    Spec {
        /// Subcommand: load, validate
        subcommand: String,
        /// Path to spec file
        path: Option<String>,
    },
    /// Incident - error analysis
    Incident {
        /// Error message or file
        error: Option<String>,
    },
    /// Integrations,
    Integrations,
    /// Context management
    Context {
        /// Subcommand: status, compact, summary, estimate
        subcommand: String,
        /// Args for estimate
        args: Vec<String>,
    },
    /// Corrections - command corrections
    Corrections {
        /// Command to correct
        command: Option<String>,
        /// Suggestions
        #[arg(long, short)]
        suggestions: Vec<String>,
    },
    /// Config - manage configuration
    Config {
        /// Subcommand: show, set, edit
        subcommand: Option<String>,
        /// Key for 'set' command
        key: Option<String>,
        /// Value for 'set' command
        value: Option<String>,
    },
    /// Chat mode
    Chat {
        /// Initial message
        message: Option<String>,
    },
    /// Run script file (.du)
    Script {
        /// Path to script file
        path: String,
        /// Arguments to pass to the script
        args: Vec<String>,
    },
    /// Daemon mode (long-running server)
    Daemon {
        /// Subcommand: start, stop, status, port
        subcommand: String,
        /// Port number (for 'start' subcommand)
        port: Option<u16>,
    },
    /// Command history
    History {
        /// Subcommand: list, search, clear
        subcommand: Option<String>,
        /// Search query
        query: Option<String>,
    },
    /// Toolchains - build, test, lint
    Toolchain {
        /// Command: build, test, lint, dev, clean, format, deps, update
        command: String,
    },
    /// Project templates/scaffolding
    Init {
        /// Template name
        template: String,
        /// Project name
        name: Option<String>,
        /// Target directory
        target: Option<String>,
    },
    /// Enterprise features
    Enterprise {
        /// Subcommand: license, config, status, enable, disable
        subcommand: String,
        /// Arguments for the subcommand
        args: Vec<String>,
    },
    /// Atomic file edit with diff/rollback
    Edit {
        /// File to edit
        file: String,
        /// Text to find
        find: String,
        /// Text to replace with
        replace: String,
    },
    /// Run tests for current project
    TestRunner {
        /// Directory to run tests in (default: current)
        directory: Option<String>,
    },
    /// Autonomous development loop (watch, test, format, lint)
    Devloop {
        /// Enable file watching
        #[arg(long)]
        watch: bool,
        /// Run tests on file change
        #[arg(long)]
        test: bool,
        /// Auto-format on change
        #[arg(long)]
        format: bool,
        /// Auto-lint on change
        #[arg(long)]
        lint: bool,
        /// Max iterations (0 = infinite)
        #[arg(long, default_value = "0")]
        iterations: usize,
    },
    /// Resolve Git merge conflicts
    Resolve {
        /// Branch with conflicts (default: current)
        branch: Option<String>,
        /// Strategy: ours, theirs, both, ai (default: ai)
        #[arg(long)]
        strategy: Option<String>,
        /// Show what would be changed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Start interactive debugger
    Dbg {
        /// Command: attach, breakpoint, watch, status, logs, repl
        #[arg(long)]
        command: Option<String>,
        /// Target (file:line for breakpoint, pid for attach)
        target: Option<String>,
    },
    /// Start live reload server with browser auto-refresh
    Reload {
        /// Port for live reload server
        #[arg(long, default_value = "35729")]
        port: u16,
        /// Root directory to serve
        #[arg(long)]
        root: Option<String>,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },
    /// GitHub automation - commit, PR, CI, releases
    Github {
        /// Subcommand: commit, push, pr, status, merge, issue, release, sync
        subcommand: String,
        /// Title for PR/issue
        title: Option<String>,
        /// Body/description
        body: Option<String>,
        /// Branch name
        branch: Option<String>,
        /// PR/Issue number (for view/merge)
        number: Option<u32>,
        /// Additional args
        args: Vec<String>,
    },
}
