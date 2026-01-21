# Waltzing Extension for Zed

This extension provides Language Server Protocol (LSP) support for [Waltzing](https://github.com/awesomike/waltzing) template files in the Zed editor.

## Features

- Real-time syntax validation for `.wtz` template files
- Error diagnostics with suggestions and code examples
- Syntax highlighting (HTML-based with template support)
- Bracket matching and auto-close

## Installation

### 1. Install the Language Server

First, build and install the Waltzing LSP server:

```bash
# From the waltzing repository root
cargo install --path lsp

# Or build and copy manually
cargo build --release -p waltzing-lsp
cp target/release/waltzing-lsp ~/.local/bin/
```

### 2. Install the Extension

#### Option A: Install as Dev Extension (Recommended for Development)

1. Open Zed
2. Open the command palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
3. Run "zed: install dev extension"
4. Select the `extensions/zed` directory from the waltzing repository

#### Option B: Build and Install

```bash
cd extensions/zed
cargo build --release --target wasm32-wasi
# Then install the built extension in Zed
```

## Configuration

You can customize the LSP binary path in your Zed settings (`~/.config/zed/settings.json`):

```json
{
  "lsp": {
    "waltzing-lsp": {
      "binary": {
        "path": "/custom/path/to/waltzing-lsp"
      }
    }
  }
}
```

## File Types

This extension handles files with the following extensions:
- `.wtz`
- `.html.wtz`
- `.css.wtz`
- `.js.wtz`

## Troubleshooting

### LSP not starting

1. Ensure `waltzing-lsp` is installed and in your PATH:
   ```bash
   which waltzing-lsp
   ```

2. Check Zed's LSP log for errors:
   - Open command palette
   - Run "zed: open log"
   - Look for "waltzing" related messages

3. Verify the binary path in settings if using a custom location

### No syntax highlighting

The extension uses HTML grammar as a base. If highlighting seems off, ensure the file has a `.wtz` extension.

## Development

To rebuild the extension after changes:

1. Make your changes to `src/lib.rs`
2. In Zed, open the extensions panel
3. Find "Waltzing" and click "Rebuild"

Or from command line:
```bash
cargo build --release --target wasm32-wasi
```

## License

MIT - Same as the Waltzing project
