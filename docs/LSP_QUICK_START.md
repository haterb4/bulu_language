# Bulu LSP Quick Start Guide

## What is the Bulu LSP?

The Bulu Language Server Protocol (LSP) implementation provides IDE features for the Bulu programming language, including:
- Auto-completion
- Go-to-definition
- Find references
- Hover documentation
- Error diagnostics
- Rename refactoring
- Code actions

## Quick Setup

### 1. Build the LSP Server

```bash
cargo build --release --bin bulu_lsp
```

The binary will be at: `target/release/bulu_lsp`

### 2. Configure Your Editor

#### VS Code (Recommended)

**Option A: Extension Officielle Bulu (Recommandé)**

Installer l'extension officielle Bulu qui inclut la coloration syntaxique et l'intégration LSP :

```bash
cd vscode-extension
npm install
./scripts/build.sh
code --install-extension bulu-language-*.vsix
```

Voir [VSCODE_EXTENSION_SUMMARY.md](../VSCODE_EXTENSION_SUMMARY.md) pour plus de détails.

**Option B: Configuration Manuelle**

Si vous préférez configurer manuellement, installez "Generic LSP Client" et ajoutez à `settings.json`:

```json
{
  "genericLsp.languageServers": [
    {
      "languageId": "bulu",
      "command": "/path/to/bulu_lsp",
      "args": [],
      "fileExtensions": ["bu"]
    }
  ]
}
```

#### Neovim

Add to your config:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.bulu_lsp then
  configs.bulu_lsp = {
    default_config = {
      cmd = {'bulu_lsp'},
      filetypes = {'bulu'},
      root_dir = lspconfig.util.root_pattern('lang.toml', '.git'),
    },
  }
end

lspconfig.bulu_lsp.setup{}
```

### 3. Test It

Open a `.bu` file and try:
- Type `func` and press Ctrl+Space for completion
- Hover over keywords to see documentation
- Use F12 to go to definition

## Features at a Glance

| Feature | Shortcut (VS Code) | Description |
|---------|-------------------|-------------|
| Completion | Ctrl+Space | Auto-complete keywords, functions, types |
| Hover | Mouse hover | Show documentation |
| Go-to-Definition | F12 | Jump to symbol definition |
| Find References | Shift+F12 | Find all uses of symbol |
| Rename | F2 | Rename symbol everywhere |
| Code Actions | Ctrl+. | Quick fixes and refactorings |

## Troubleshooting

**LSP not starting?**
- Check that `bulu_lsp` is in your PATH
- Test manually: `bulu_lsp` (should wait for input)

**No completions?**
- Ensure file has `.bu` extension
- Save the file first

**Errors not showing?**
- Save the file to trigger analysis

## More Information

See the full [LSP Guide](LSP_GUIDE.md) for detailed documentation.
