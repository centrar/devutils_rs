# DevUtils Development Guide

## Commands

```bash
# Build
cargo build --release

# Run tests
cargo test --all

# Run with args
./target/release/devutils --help

# Lint
cargo clippy --all -- -D warnings

# Format
cargo fmt --all

# Benchmark
./target/release/devutils benchmark
```

## Key Files

- `src/main.rs` - CLI entry point
- `src/ai.rs` - AI features (OpenAI integration)
- `src/search.rs` - File search, grep
- `src/ui.rs` - Interactive TUI
- `src/git.rs` - Git utilities

## Features

- AI code generation (requires OPENAI_API_KEY)
- Semantic search with embeddings
- Interactive TUI with ratatui
- Cross-platform support