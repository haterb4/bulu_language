# Bulu Language Server Protocol (LSP) Guide

## Overview

The Bulu Language Server provides IDE integration for the Bulu programming language through the Language Server Protocol (LSP). This enables features like auto-completion, go-to-definition, hover documentation, and more in any LSP-compatible editor.

## Features

### Implemented Features

1. **Syntax Highlighting and Error Reporting**
   - Real-time lexical and syntax error detection
   - Clear error messages with line and column information
   - Diagnostic severity levels (Error, Warning)

2. **Code Completion**
   - Keyword completion (if, else, func, struct, etc.)
   - Built-in function completion (print, len, make, etc.)
   - Type completion (int32, string, bool, etc.)
   - Context-aware completions (member access, imports)
   - Snippet support for functions

3. **Hover Information**
   - Documentation for keywords
   - Function signatures for built-ins
   - Type information
   - Markdown-formatted hover content

4. **Go-to-Definition**
   - Navigate to function definitions
   - Navigate to struct definitions
   - Navigate to variable declarations

5. **Find References**
   - Find all references to a symbol
   - Works for functions, structs, and variables

6. **Symbol Navigation**
   - Document symbols (outline view)
   - Workspace symbols (search across files)
   - Symbol kinds: Function, Struct, Variable

7. **Rename Refactoring**
   - Rename symbols across the document
   - Updates all references automatically

8. **Code Actions (Quick Fixes)**
   - Add missing import statements
   - Remove unused variables
   - Extract function (command-based)
   - Inline variable (command-based)

9. **Signature Help**
   - Parameter information while typing
   - Function signature display
   - Active parameter highlighting

## Installation

### Building from Source

```bash
# Build the LSP server
cargo build --release --bin bulu_lsp

# The binary will be at: target/release/bulu_lsp
```

### Installing Globally

```bash
# Install to cargo bin directory
cargo install --path . --bin bulu_lsp

# Or copy to a directory in your PATH
sudo cp target/release/bulu_lsp /usr/local/bin/
```

## Editor Configuration

### Visual Studio Code

Create a VS Code extension or use a generic LSP client:

1. Install the "Generic LSP Client" extension
2. Add to your `settings.json`:

```json
{
  "genericLsp.languageServers": [
    {
      "languageId": "bulu",
      "command": "bulu_lsp",
      "args": [],
      "fileExtensions": ["bu"]
    }
  ]
}
```

### Neovim (with nvim-lspconfig)

Add to your Neovim configuration:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define Bulu LSP
if not configs.bulu_lsp then
  configs.bulu_lsp = {
    default_config = {
      cmd = {'bulu_lsp'},
      filetypes = {'bulu'},
      root_dir = lspconfig.util.root_pattern('lang.toml', '.git'),
      settings = {},
    },
  }
end

-- Setup Bulu LSP
lspconfig.bulu_lsp.setup{}

-- Set filetype for .bu files
vim.cmd([[
  autocmd BufRead,BufNewFile *.bu set filetype=bulu
]])
```

### Emacs (with lsp-mode)

Add to your Emacs configuration:

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(bulu-mode . "bulu"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "bulu_lsp")
                  :major-modes '(bulu-mode)
                  :server-id 'bulu-lsp))

(add-hook 'bulu-mode-hook #'lsp)
```

### Sublime Text (with LSP package)

Add to your LSP settings:

```json
{
  "clients": {
    "bulu": {
      "enabled": true,
      "command": ["bulu_lsp"],
      "selector": "source.bulu",
      "languageId": "bulu"
    }
  }
}
```

## Usage

Once configured, the LSP server will automatically start when you open a `.bu` file. The following features will be available:

### Auto-Completion

Type and press `Ctrl+Space` (or your editor's completion trigger) to see:
- Keywords (if, func, struct, etc.)
- Built-in functions (print, len, make, etc.)
- Types (int32, string, bool, etc.)
- Context-specific suggestions

### Hover Documentation

Hover over any keyword, function, or type to see:
- Documentation
- Function signatures
- Type information

### Go-to-Definition

Place cursor on a symbol and use your editor's "Go to Definition" command:
- VS Code: `F12` or `Ctrl+Click`
- Neovim: `gd` (with default LSP keybindings)
- Emacs: `M-.`

### Find References

Use your editor's "Find References" command:
- VS Code: `Shift+F12`
- Neovim: `gr` (with default LSP keybindings)
- Emacs: `M-?`

### Rename Symbol

Place cursor on a symbol and use your editor's "Rename" command:
- VS Code: `F2`
- Neovim: `<leader>rn` (with default LSP keybindings)
- Emacs: `M-x lsp-rename`

### Code Actions

When diagnostics appear, use your editor's "Code Actions" command:
- VS Code: `Ctrl+.`
- Neovim: `<leader>ca` (with default LSP keybindings)
- Emacs: `M-x lsp-execute-code-action`

## Architecture

The LSP server is built using:
- **tower-lsp**: LSP protocol implementation
- **tokio**: Async runtime
- **dashmap**: Concurrent document storage

### Components

1. **Backend** (`src/lsp/backend.rs`)
   - Main LSP server implementation
   - Document state management
   - Request routing

2. **Diagnostics** (`src/lsp/diagnostics.rs`)
   - Real-time error detection
   - Lexical and syntax analysis
   - Error-to-diagnostic conversion

3. **Completion** (`src/lsp/completion.rs`)
   - Keyword completions
   - Built-in function completions
   - Type completions
   - Context-aware suggestions

4. **Hover** (`src/lsp/hover.rs`)
   - Hover information provider
   - Signature help
   - Documentation formatting

5. **Navigation** (`src/lsp/navigation.rs`)
   - Go-to-definition
   - Find references
   - Document symbols

6. **Refactor** (`src/lsp/refactor.rs`)
   - Rename refactoring
   - Code actions
   - Quick fixes

## Testing

Run the LSP tests:

```bash
cargo test --test lsp_tests
```

All tests should pass, covering:
- Initialization
- Completion providers
- Hover information
- Symbol navigation
- Refactoring operations

## Troubleshooting

### LSP Server Not Starting

1. Check that `bulu_lsp` is in your PATH:
   ```bash
   which bulu_lsp
   ```

2. Test the server manually:
   ```bash
   bulu_lsp
   ```
   The server should start and wait for LSP messages on stdin.

3. Check editor logs for error messages

### No Completions Appearing

1. Verify the file has `.bu` extension
2. Check that the LSP client is configured for `bulu` language
3. Ensure the document is saved (some editors require this)

### Diagnostics Not Updating

1. Save the file to trigger re-analysis
2. Check that the LSP server is running
3. Verify no syntax errors in the LSP configuration

## Future Enhancements

Planned features for future versions:
- Semantic token highlighting
- Inlay hints for type information
- Code lens for test running
- Workspace-wide symbol search
- Import organization
- Format on save integration
- Debugging protocol support

## Contributing

To contribute to the LSP implementation:

1. Read the LSP specification: https://microsoft.github.io/language-server-protocol/
2. Check existing issues and feature requests
3. Add tests for new features
4. Ensure all tests pass before submitting PR

## License

The Bulu LSP server is part of the Bulu project and is licensed under the MIT License.
