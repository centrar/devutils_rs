# 🚀 DevUtils - The Ultimate AI-Powered Developer Toolkit

**The fastest all-in-one CLI for developers who want everything working NOW.**

[![Crates.io](https://img.shields.io/crates/v/devutils.svg)](https://crates.io/crates/devutils)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/devutils/devutils/workflows/CI/badge.svg)](https://github.com/devutils/devutils/actions)

## Why DevUtils?

Tired of juggling 10 different tools? DevUtils combines **AI autonomy**, **lightning-fast search**, **git workflows**, **cloud sync**, and **plugin ecosystem** into one CLI that actually gets things done.

### 🔥 Key Features

- **🤖 AI Autonomous Agent** - "Fix all failing tests" → AI diagnoses, fixes, and commits
- **⚡ Parallel Search** - ripgrep-speed file searching with multi-threading
- **🔌 Plugin System** - 100+ plugins for every workflow
- **☁️ Cloud Sync** - Sync config/plugins across devices (Dropbox, GDrive, S3)
- **🎨 50+ Commands** - Git, AI, search, build, test, lint - everything in one place
- **🖥️ Interactive TUI** - Beautiful lazygit-style interface
- **📦 Project Templates** - Scaffold new projects instantly

## Installation

### From crates.io (Recommended)

```bash
cargo install devutils
```

### From source

```bash
git clone https://github.com/devutils/devutils.git
cd devutils
cargo build --release
```

## Quick Start

### 1. AI Autonomous Agent

```bash
# Let AI fix your tests
devutils autonomous "Fix the failing tests"

# Add a new feature
devutils autonomous "Add error handling to all API calls"
```

### 2. Parallel Search

```bash
# Lightning-fast search
devutils search "authentication" --parallel

# Search with regex
devutils grep "TODO.*FIXME" --parallel
```

### 3. Plugin System

```bash
# Install plugins
devutils plugin install git-clean
devutils plugin install docker-helper

# List installed
devutils plugin list

# Run a plugin
devutils plugin run git-clean
```

### 4. Project Templates

```bash
# Create a new Rust CLI project
devutils init rust-cli myapp

# Create a Python project
devutils init python-cli mypyapp
```

## Commands Overview

### AI Commands
- `devutils autonomous <task>` - AI autonomous agent
- `devutils generate <prompt>` - Generate code
- `devutils explain` - Explain code
- `devutils debug` - Debug errors

### Search Commands  
- `devutils search <pattern>` - Search codebase
- `devutils find <pattern>` - Find files
- `devutils grep <pattern>` - Grep search
- `devutils search --parallel` - Parallel search (fast!)

### Git Commands
- `devutils status` - Git status
- `devutils pick branch` - Pick branch (fuzzy)
- `devutils pick commit` - Pick commit

### Development
- `devutils toolchain build` - Build project
- `devutils toolchain test` - Run tests
- `devutils toolchain lint` - Lint code
- `devutils init <template>` - Create project

### System
- `devutils config` - Manage configuration
- `devutils history` - Command history
- `devutils repl` - Interactive REPL
- `devutils daemon` - Background daemon

## Configuration

DevUtils uses `~/.devutils.toml` for configuration:

```toml
default_ai_provider = "openai"
default_ai_model = "gpt-4"
editor = "code"
shell = "bash"
keybindings = "vim"
color_theme = "dark"
```

## Plugins

DevUtils has a growing ecosystem of plugins:

```bash
# Search marketplace
devutils plugin search docker

# Install
devutils plugin install <name>

# Run
devutils plugin run <name>
```

## Performance

- **Parallel Search**: 10x faster than standard grep
- **Startup Time**: <200ms (optimized with LTO)
- **Binary Size**: ~7MB (statically linked)
- **Memory Usage**: <50MB for most operations

## Roadmap

- [ ] Local AI (Ollama/llama.cpp integration)
- [ ] Semantic code search
- [ ] VS Code extension
- [ ] More workflow templates
- [ ] Team collaboration features

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Development setup
git clone https://github.com/devutils/devutils.git
cargo build
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with love using:
- [clap](https://github.com/clap-rs/clap) - CLI framework
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- And many more amazing crates!

---

**Made with ❤️ for developers, by developers.**

[Website](https://devutils.ai) | [Documentation](https://docs.devutils.ai) | [Discord](https://discord.gg/devutils)
