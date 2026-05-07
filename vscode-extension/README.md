# DevUtils - VS Code Extension

AI-powered autonomous developer CLI integration for Visual Studio Code.

## Features

- 🤖 **Autonomous Agent**: Run AI agent tasks directly from VS Code
- 💬 **Chat**: Chat with AI about your code
- ✨ **Generate Code**: Generate code from natural language
- 📖 **Explain Code**: Get explanations for selected code
- 🔧 **Fix Code**: Automatically fix errors in selected code

## Installation

1. **From VSIX** (Development):
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   # In VS Code: Extensions → ⋯ → Install from VSIX → select devutils-1.0.0.vsix
   ```

2. **From Marketplace** (Coming soon):
   - Open VS Code
   - Extensions (Ctrl+Shift+X)
   - Search for "DevUtils"
   - Install

## Usage

### Commands

Open Command Palette (Ctrl+Shift+P) and search for:

- `DevUtils: Run Autonomous Agent` - Run autonomous AI agent
- `DevUtils: Chat with AI` - Chat with AI
- `DevUtils: Generate Code` - Generate code from prompt
- `DevUtils: Explain Code` - Explain selected code
- `DevUtils: Fix Errors` - Fix selected code

### Context Menu

- Right-click in editor with text selected → `DevUtils: Explain Code`
- Right-click in editor with text selected → `DevUtils: Fix Errors`
- Right-click in Explorer → `DevUtils: Run Autonomous Agent`

### Output Panel

All DevUtils output appears in the "DevUtils" output panel (View → Output → DevUtils).

## Configuration

Open Settings (Ctrl+,) and search for "DevUtils":

- `devutils.apiKey`: Your OpenAI API key (or set `OPENAI_API_KEY` env var)
- `devutils.provider`: AI provider (openai, nvidia, deepseek, ollama)
- `devutils.verbose`: Enable verbose output

## Requirements

- DevUtils CLI installed (`devutils` command available in PATH)
- OpenAI API key (or other supported AI provider)

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch

# Lint
npm run lint
```

## License

MIT

## Support

- GitHub: https://github.com/devutils/devutils
- Website: https://devutils.ai
