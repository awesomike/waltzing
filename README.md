# Waltzing Editor Support

This repository contains editor integrations and tooling for [Waltzing](https://waltzing.awesomike.com), a compile-time template engine for Rust.

## Contents

- **tree-sitter/** - Tree-sitter grammar for Waltzing template syntax
- **extensions/** - Editor extensions
  - **extensions/zed/** - Zed editor extension with LSP support

## Tree-sitter Grammar

The Tree-sitter grammar provides syntax highlighting for `.wtz` files. It can be used with any editor that supports Tree-sitter.

### Neovim Setup

Add to your `init.lua` or nvim-treesitter config:

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.waltzing = {
    install_info = {
        url = "https://github.com/awesomike/waltzing",
        files = { "src/parser.c" },
        branch = "main",
        location = "tree-sitter",
    },
    filetype = "waltzing",
}

vim.filetype.add({
    extension = {
        wtz = "waltzing",
    },
})
```

Then install with `:TSInstall waltzing`

### Building from Source

```bash
cd tree-sitter
npm install
npx tree-sitter generate
npx tree-sitter build-wasm
```

## Zed Extension

The Zed extension provides:
- Syntax highlighting via Tree-sitter grammar
- LSP support via `waltzing-lsp`
- File association for `.wtz` files

### Installation

1. Open Zed
2. Open Extensions (Cmd+Shift+X on macOS)
3. Search for "Waltzing"
4. Click Install

### Manual Installation

```bash
git clone https://github.com/awesomike/waltzing
cd waltzing/extensions/zed
zed --install-extension .
```

## Language Server (LSP)

The Waltzing Language Server provides IDE features across all editors that support LSP:
- Real-time diagnostics
- Go to definition
- Hover information
- Auto-completion
- Document symbols

### Installation

```bash
curl -fsSL https://waltzing.awesomike.com/install | bash -s -- --binary lsp
```

## Documentation

- [Editor Setup Guide](https://waltzing.awesomike.com/docs/editors)
- [Main Documentation](https://waltzing.awesomike.com/docs)
- [Getting Started](https://waltzing.awesomike.com/docs/getting-started)

## License

MIT
