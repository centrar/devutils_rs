//! DevUtils - The fastest AI-powered developer toolkit
//! High-performance Rust CLI with Autonomous Agentic capabilities.

use std::process::ExitCode;
use std::time::Instant;
use devutils::cli_types::{Cli, Commands};
use devutils::ultimate_agent::AutonomousAgent;
use devutils::completions;
use clap::Parser;

fn main() -> ExitCode {
    let start = Instant::now();
    let args = Cli::parse();

    if let Some(shell) = &args.completions {
        match completions::generate_completions(shell) {
            Ok(s) => {
                println!("{}", s);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    match run(&args) {
        Ok(_) => {
            if args.verbose {
                eprintln!("Completed in {:.2?}", start.elapsed());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        Some(Commands::Version) => {
            println!("devutils 1.0.0 (Industrial)");
        }
        Some(Commands::Agent { task, verbose }) => {
            let task_str = task.join(" ");
            let mut agent = match AutonomousAgent::new(*verbose) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("❌ Failed to initialize agent: {}", e);
                    return Err(e.into());
                }
            };
            agent.verbose = *verbose;
            
            match agent.execute(&task_str) {
                Ok(_) => {
                    agent.finalize(true);
                    println!("✅ Agent completed task.");
                }
                Err(e) => {
                    agent.finalize(false);
                    return Err(e.into());
                }
            }
        }
        Some(Commands::Completions { shell }) => {
            match completions::generate_completions(shell) {
                Ok(s) => println!("{}", s),
                Err(e) => return Err(e.into()),
            }
        }
        // Plugin management commands
        Some(Commands::Plugin { subcommand, name, args }) => {
            handle_plugin_command(subcommand, name.as_deref(), args)?;
        }
        // Marketplace commands
        Some(Commands::Marketplace { subcommand, query }) => {
            handle_marketplace_command(subcommand, query.as_deref())?;
        }
        // ── Git ─────────────────────────────────────────────────────────────
        Some(Commands::Status) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Status)?;
        }
        Some(Commands::Commits) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Commits { n: 10 })?;
        }
        Some(Commands::Branches) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Branches)?;
        }
        Some(Commands::Commit { message }) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Commit { message: message.clone() })?;
        }
        Some(Commands::Push) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Push)?;
        }
        Some(Commands::Ignore) => {
            devutils::git::GitOps::new().run(devutils::git::GitCommands::Ignore { template: None })?;
        }
        // ── Search ──────────────────────────────────────────────────────────
        Some(Commands::Search { pattern, parallel, case_sensitive, max_results, .. }) => {
            let grep = devutils::search::Grep::new(".");
            let results = if *parallel {
                grep.search_parallel(pattern, !case_sensitive, false)
            } else {
                grep.search(pattern, !case_sensitive, false)
            };
            if results.is_empty() {
                println!("No matches found for '{}'", pattern);
            } else {
                println!("\x1b[36m🔍 {} match(es) for '{}'\x1b[0m\n", results.len().min(*max_results), pattern);
                for (file, line, content) in results.iter().take(*max_results) {
                    println!("  \x1b[33m{}:{}\x1b[0m  {}", file, line, content.trim());
                }
            }
        }
        Some(Commands::Find { pattern }) => {
            let results = devutils::search::FileSearch::new(".").find_parallel(pattern, None, true);
            if results.is_empty() {
                println!("No files found matching '{}'", pattern);
            } else {
                println!("\x1b[36m📁 {} file(s) found\x1b[0m\n", results.len());
                for f in results.iter().take(100) { println!("  {}", f); }
            }
        }
        Some(Commands::Grep { pattern }) => {
            let results = devutils::search::Grep::new(".").search_parallel(pattern, true, false);
            for (file, line, content) in results.iter().take(200) {
                println!("{}:{}: {}", file, line, content.trim());
            }
        }
        // ── System ──────────────────────────────────────────────────────────
        Some(Commands::System) => {
            println!("{}", devutils::quick::run_quick_command("sys"));
        }
        Some(Commands::LocalIp) => {
            use std::net::UdpSocket;
            let ip = UdpSocket::bind("0.0.0.0:0")
                .and_then(|s| { s.connect("8.8.8.8:80")?; s.local_addr() })
                .map(|a| a.ip().to_string())
                .unwrap_or_else(|_| "127.0.0.1".to_string());
            println!("🌐 Local IP: {}", ip);
        }
        Some(Commands::Port { port }) => {
            use std::net::TcpListener;
            match TcpListener::bind(format!("0.0.0.0:{}", port)) {
                Ok(_) => println!("✅ Port {} is available", port),
                Err(_) => println!("❌ Port {} is in use", port),
            }
        }
        // ── All-in-One Utilities ─────────────────────────────────────────────
        Some(Commands::Utils { name, args }) => {
            println!("{}", devutils::all_utils::run_utility(name, args));
        }
        Some(Commands::Pipe { pipeline }) => {
            println!("{}", devutils::all_utils::run_pipeline(pipeline));
        }
        // ── Quick ────────────────────────────────────────────────────────────
        Some(Commands::Quick { command }) => {
            println!("{}", devutils::quick::run_quick_command(command));
        }
        // ── Toolchain (build / test / lint / format) ─────────────────────────
        Some(Commands::Toolchain { command }) => {
            let (prog, cmd_args): (&str, &[&str]) = match command.as_str() {
                "build"  => ("cargo", &["build"]),
                "test"   => ("cargo", &["test"]),
                "lint"   => ("cargo", &["clippy"]),
                "format" => ("cargo", &["fmt"]),
                "clean"  => ("cargo", &["clean"]),
                "dev"    => ("cargo", &["run"]),
                _ => { println!("Unknown toolchain command: {}", command); return Ok(()); }
            };
            let status = std::process::Command::new(prog).args(cmd_args).status()?;
            if !status.success() { return Err("Toolchain command failed".into()); }
        }
        // ── Terminal UI (ratatui) ────────────────────────────────────────────
        Some(Commands::Tui { .. }) | Some(Commands::Interactive) => {
            if let Err(e) = devutils::tui::run_tui() {
                eprintln!("Failed to launch TUI: {}", e);
            }
        }
        // ── SOTA Integrations ────────────────────────────────────────────────
        Some(Commands::Resolve { branch, strategy, dry_run }) => {
            println!("{}", devutils::merge_resolver::run_merge_resolve(branch.as_deref(), strategy.as_deref(), *dry_run)?);
        }
        Some(Commands::Offline { subcommand, prompt }) => {
            match subcommand.as_str() {
                "status" => println!("{}", devutils::local_models::get_status()),
                "install" => { devutils::local_models::install_ollama()?; println!("Ollama installed successfully."); }
                _ => println!("{}", devutils::local_models::run_local(prompt.as_deref().unwrap_or("Hello"))?),
            }
        }
        Some(Commands::Devloop { watch, test, format, lint, iterations }) => {
            let mut loop_dev = devutils::dev_loop::DevLoop::new(std::path::Path::new("."));
            loop_dev = loop_dev.with_config(devutils::dev_loop::DevLoopConfig {
                watch: *watch, test_on_change: *test, format_on_change: *format, lint_on_change: *lint, auto_deps: true, build_on_change: true, poll_interval_ms: 500
            });
            let iters = if *iterations == 0 { None } else { Some(*iterations) };
            loop_dev.run(iters)?;
        }
        Some(Commands::Github { subcommand, title, branch, number, .. }) => {
            match subcommand.as_str() {
                "pr" => {
                    let ai_client = devutils::ai::AIClient::new();
                    println!("{}", devutils::github::full_auto_pr(&ai_client, title.as_deref().unwrap_or("Auto PR"), branch.as_deref())?);
                }
                "merge" => {
                    let num = number.ok_or_else(|| "Error: PR number is required for merge".to_string())?;
                    println!("{}", devutils::github::merge_pr(num, None, None)?);
                }
                "status" => println!("{}", devutils::github::check_ci()?),
                "sync" => { devutils::github::sync_with_remote(branch.as_deref().unwrap_or("master"))?; println!("Synced successfully"); }
                _ => println!("Github subcommand '{}' is not fully wired yet.", subcommand),
            }
        }
        // ── Catch-all ────────────────────────────────────────────────────────
        Some(cmd) => {
            eprintln!("Command '{:?}' is not yet implemented in this build.", cmd);
            eprintln!("Run 'devutils --help' for available commands.");
            return Err("Unknown command".into());
        }
        None => {
            println!("DevUtils v1.0.0\nRun 'devutils --help' for usage.");
        }
    }
    Ok(())
}

fn handle_plugin_command(subcommand: &str, name: Option<&str>, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use devutils::unified_plugins::UnifiedPluginSystem;
    
    let mut system = UnifiedPluginSystem::new();
    
    match subcommand {
        "list" | "ls" => {
            let installed = system.list_installed();
            if installed.is_empty() {
                println!("\n\x1b[33m📦 No plugins installed\x1b[0m\n");
                println!("Install plugins with: devutils plugin install <name>");
                println!("Browse marketplace: devutils marketplace list\n");
            } else {
                println!("\n\x1b[36m📦 Installed Plugins\x1b[0m\n");
                for (i, plugin) in installed.iter().enumerate() {
                    println!("  {}. \x1b[32m{}\x1b[0m v{}", i + 1, plugin.name, plugin.version);
                    println!("     {}", plugin.description);
                    println!("     \x1b[90mby {}\x1b[0m\n", plugin.author);
                }
            }
        }
        "install" => {
            if let Some(name) = name {
                match system.install(name) {
                    Ok(msg) => println!("\x1b[32m✓\x1b[0m {}", msg),
                    Err(e) => println!("\x1b[31m✗ Error:\x1b[0m {}", e),
                }
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils plugin install <name>");
            }
        }
        "uninstall" | "remove" => {
            if let Some(name) = name {
                match system.uninstall(name) {
                    Ok(msg) => println!("\x1b[32m✓\x1b[0m {}", msg),
                    Err(e) => println!("\x1b[31m✗ Error:\x1b[0m {}", e),
                }
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils plugin uninstall <name>");
            }
        }
        "run" | "exec" | "execute" => {
            if let Some(name) = name {
                // Check if it's a native plugin first
                if system.is_native(name) {
                    match system.execute_native(name, args) {
                        Ok(output) => println!("{}", output),
                        Err(e) => println!("\x1b[31m✗ Error:\x1b[0m {}", e),
                    }
                } else {
                    // Try to execute as installed plugin
                    let command = args.get(0).map(|s| s.as_str()).unwrap_or("default");
                    let plugin_args: Vec<String> = args.iter().skip(1).cloned().collect();
                    match system.execute(name, command, &plugin_args) {
                        Ok(output) => println!("{}", output),
                        Err(e) => println!("\x1b[31m✗ Error:\x1b[0m {}", e),
                    }
                }
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils plugin run <name> [args...]");
            }
        }
        "info" => {
            if let Some(name) = name {
                if let Some(plugin) = system.get(name) {
                    println!("\n\x1b[36m📦 {}\x1b[0m v{}", plugin.name, plugin.version);
                    println!("\n{}\n", plugin.description);
                    println!("\x1b[90mCategory:\x1b[0m {}", plugin.category.as_str());
                    println!("\x1b[90mAuthor:\x1b[0m {}", plugin.author);
                    println!("\x1b[90mDownloads:\x1b[0m {}", plugin.downloads);
                    if let Some(homepage) = plugin.homepage {
                        println!("\x1b[90mHomepage:\x1b[0m {}", homepage);
                    }
                    if plugin.featured {
                        println!("\n\x1b[32m✨ Featured Plugin\x1b[0m");
                    }
                    if !plugin.tags.is_empty() {
                        println!("\n\x1b[90mTags:\x1b[0m {}", plugin.tags.join(", "));
                    }
                    println!();
                } else {
                    println!("\x1b[31m✗\x1b[0m Plugin '{}' not found in registry", name);
                }
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils plugin info <name>");
            }
        }
        "native" => {
            println!("\n\x1b[36m⚡ Native Plugins (100 total)\x1b[0m\n");
            println!("These plugins are built-in and require no installation:\n");
            for (i, name) in system.list_native().iter().enumerate() {
                println!("  {}. {}", i + 1, name);
            }
            println!("\n\x1b[90mUsage:\x1b[0m devutils plugin run <name> [args...]");
            println!("\x1b[90mExample:\x1b[0m devutils plugin run httpie https://api.github.com GET\n");
        }
        _ => {
            println!("\x1b[33mUsage:\x1b[0m devutils plugin <subcommand> [options]");
            println!("\nSubcommands:");
            println!("  list              List installed plugins");
            println!("  install <name>    Install a plugin");
            println!("  uninstall <name>  Uninstall a plugin");
            println!("  run <name> [args] Execute a plugin");
            println!("  info <name>       Show plugin information");
            println!("  native            List native plugins");
        }
    }
    
    Ok(())
}

fn handle_marketplace_command(subcommand: &str, query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use devutils::unified_plugins;
    use devutils::registry::PluginCategory;
    
    match subcommand {
        "list" | "ls" => {
            unified_plugins::display_all_plugins();
        }
        "search" => {
            if let Some(query) = query {
                unified_plugins::display_search_results(query);
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils marketplace search <query>");
            }
        }
        "featured" => {
            let system = devutils::unified_plugins::UnifiedPluginSystem::new();
            println!("\n\x1b[36m✨ Featured Plugins\x1b[0m\n");
            for (i, plugin) in system.featured().iter().enumerate() {
                println!("  {}. \x1b[1m{}\x1b[0m", i + 1, plugin.name);
                println!("     {}", plugin.description);
                println!("     \x1b[90m{} downloads | v{} | by {}\x1b[0m\n", 
                    format_downloads(plugin.downloads), plugin.version, plugin.author);
            }
        }
        "categories" => {
            unified_plugins::display_all_categories();
        }
        "category" => {
            if let Some(cat_name) = query {
                // Try to parse category from string
                let category = match cat_name.to_lowercase().as_str() {
                    "version-control" | "vcs" => PluginCategory::VersionControl,
                    "cloud" | "cloud-providers" => PluginCategory::Cloud,
                    "containers" | "container" => PluginCategory::Containers,
                    "iac" | "infrastructure" => PluginCategory::InfrastructureAsCode,
                    "ci" | "cicd" | "ci-cd" => PluginCategory::CICD,
                    "languages" | "lang" => PluginCategory::Languages,
                    "frameworks" | "framework" => PluginCategory::Frameworks,
                    "databases" | "database" | "db" => PluginCategory::Databases,
                    "observability" | "monitoring" => PluginCategory::Observability,
                    "security" | "secrets" => PluginCategory::Security,
                    "ai" | "llm" | "ml" => PluginCategory::AI,
                    "package" | "packages" | "registry" => PluginCategory::PackageRegistries,
                    "dev-tools" | "devtools" | "tools" => PluginCategory::DevTools,
                    "http" | "api" | "apis" => PluginCategory::HTTP,
                    "data" | "etl" => PluginCategory::DataETL,
                    "system" | "utils" => PluginCategory::SystemUtils,
                    "networking" | "network" => PluginCategory::Networking,
                    "mobile" => PluginCategory::Mobile,
                    "testing" | "test" => PluginCategory::Testing,
                    "docs" | "documentation" => PluginCategory::Documentation,
                    "notifications" | "integrations" => PluginCategory::Notifications,
                    "enterprise" | "compliance" => PluginCategory::Enterprise,
                    "terminal" | "ui" => PluginCategory::Terminal,
                    _ => {
                        println!("\x1b[31m✗\x1b[0m Unknown category: {}", cat_name);
                        println!("\nUse 'devutils marketplace categories' to see all categories");
                        return Ok(());
                    }
                };
                unified_plugins::display_category(&category);
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils marketplace category <name>");
            }
        }
        "stats" => {
            unified_plugins::display_stats();
        }
        "top" => {
            let system = devutils::unified_plugins::UnifiedPluginSystem::new();
            let top = system.top(20);
            println!("\n\x1b[36m🔥 Top 20 Plugins by Downloads\x1b[0m\n");
            for (i, plugin) in top.iter().enumerate() {
                println!("  {}. \x1b[1m{}\x1b[0m", i + 1, plugin.name);
                println!("     {}", plugin.description);
                println!("     \x1b[90m{} downloads | v{}\x1b[0m\n", 
                    format_downloads(plugin.downloads), plugin.version);
            }
        }
        "install" => {
            if let Some(name) = query {
                let mut system = devutils::unified_plugins::UnifiedPluginSystem::new();
                match system.install(name) {
                    Ok(msg) => println!("\x1b[32m✓\x1b[0m {}", msg),
                    Err(e) => println!("\x1b[31m✗ Error:\x1b[0m {}", e),
                }
            } else {
                println!("\x1b[33mUsage:\x1b[0m devutils marketplace install <name>");
            }
        }
        _ => {
            println!("\x1b[33mUsage:\x1b[0m devutils marketplace <subcommand> [options]");
            println!("\nSubcommands:");
            println!("  list              Browse all plugins");
            println!("  search <query>    Search for plugins");
            println!("  featured          Show featured plugins");
            println!("  categories        List all categories");
            println!("  category <name>   Show plugins in a category");
            println!("  stats             Show marketplace statistics");
            println!("  top               Show top plugins by downloads");
            println!("  install <name>    Install a plugin");
        }
    }
    
    Ok(())
}

fn format_downloads(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
