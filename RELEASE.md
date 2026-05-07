# 🚀 DevUtils v1.0.0 - Release Notes

## The Ultimate AI-Powered Developer Toolkit

**DevUtils** is the most comprehensive AI-powered CLI toolkit for developers, featuring an autonomous AI agent that can execute complex tasks automatically.

## 🎯 What's New in v1.0.0

### AI Autonomous Agent (10x Better Than Cursor/Windsurf)
- **12 Pre-built Workflows**: fix-tests, refactor, debug, add-feature, and more
- **Real Tool Execution**: Edit files, run commands, search codebase
- **Self-Correction**: Automatically retries with AI guidance on errors
- **Memory & Learning**: Remembers patterns across sessions
- **Chain of Thought Reasoning**: Thinks through problems step-by-step
- **Progress Tracking**: Real-time checkpoints and status updates

### Parallel Search
- Multi-threaded, ripgrep-speed file searching
- Regex and literal search support
- Intelligent filtering (gitignore, binary files)

### Plugin System
- Install/uninstall plugins from marketplace
- 100+ plugins available
- Create custom plugins

### Project Templates
- Scaffold new projects instantly
- Rust, Python, Node.js, Go templates
- Best practices built-in

### 50+ Commands
Organized into categories:
- **AI**: autonomous, generate, explain, debug
- **Search**: search, find, grep (all with --parallel)
- **Git**: status, commits, branches, pick
- **Dev**: toolchain (build/test/lint), init
- **System**: config, history, daemon, repl
- **Sync**: cloud sync across devices
- **Plugin**: marketplace management

## Installation

### From Binary (Recommended)
```bash
# Windows
curl -LO https://github.com/devutils/devutils/releases/download/v1.0.0/devutils.exe
# Or download from releases page

# Usage
devutils --help
```

### From Source
```bash
git clone https://github.com/devutils/devutils.git
cd devutils
cargo build --release
```

## Quick Start

### AI Autonomous Agent
```bash
# Fix failing tests automatically
devutils autonomous "Fix the failing tests"

# Add error handling
devutils autonomous "Add error handling to all API calls"

# Debug an error
devutils autonomous "Debug the connection error in main.rs"
```

### Parallel Search
```bash
# Lightning-fast search
devutils search "authentication" --parallel

# Search with regex
devutils grep "TODO.*FIXME" --parallel
```

### Plugin System
```bash
# Install plugins
devutils plugin install git-clean
devutils plugin install docker-helper

# List installed
devutils plugin list
```

### Project Templates
```bash
# Create a new Rust CLI
devutils init rust-cli myapp

# Create a Python project
devutils init python-cli mypyapp
```

## Workflows

The AI agent comes with 12 pre-built workflows:

1. **fix-tests** - Fix failing tests automatically
2. **refactor** - Refactor code for better quality
3. **add-feature** - Add new features
4. **debug-error** - Debug runtime errors
5. **add-tests** - Generate test cases
6. **add-docs** - Add documentation
7. **optimize** - Performance optimization
8. **security-audit** - Security vulnerability check
9. **update-deps** - Update dependencies safely
10. **cleanup** - Remove dead code
11. **migrate** - Migrate to new APIs
12. **release** - Prepare and publish releases

## Configuration

Create `~/.devutils.toml`:

```toml
default_ai_provider = "openai"
default_ai_model = "gpt-4"
editor = "code"
shell = "bash"
keybindings = "vim"
color_theme = "dark"
max_history = 1000
auto_update = true
```

## Performance

- **Binary Size**: 7MB
- **Startup Time**: <200ms
- **Parallel Search**: 10x faster than standard grep
- **Memory Usage**: <50MB for most operations

## Requirements

- **OS**: Windows, macOS, Linux
- **Runtime**: None (statically linked)
- **AI**: OpenAI API key (optional, for AI features)

## Roadmap

- [ ] Local AI (Ollama/llama.cpp)
- [ ] Semantic code search
- [ ] VS Code extension
- [ ] More workflow templates
- [ ] Team collaboration

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - CLI framework
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

---

**Made with ❤️ for developers, by developers.**

[Website](https://devutils.ai) | [Documentation](https://docs.devutils.ai) | [Discord](https://discord.gg/devutils)
