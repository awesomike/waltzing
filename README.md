# Waltzing

**Compile-time template engine for Rust with AI-powered development.**

Waltzing transforms `.wtz` template files into type-safe Rust code at build time. It features React-like components, MCP-powered AI development, and full editor integration.

## Quick Install

```bash
curl -fsSL https://waltzing.awesomike.com/install | bash
```

Or download individual binaries below.

## Downloads

### waltzing (CLI Compiler)

Compiles `.wtz` templates to Rust code.

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [waltzing-darwin-aarch64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-darwin-aarch64) |
| macOS (Intel) | [waltzing-darwin-x86_64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-darwin-x86_64) |
| Linux (ARM64) | [waltzing-linux-aarch64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-linux-aarch64) |
| Linux (x86_64) | [waltzing-linux-x86_64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-linux-x86_64) |

### waltzing-lsp (Language Server)

LSP server for editor integration with autocompletion and diagnostics.

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [waltzing-lsp-darwin-aarch64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-lsp-darwin-aarch64) |
| Linux (x86_64) | [waltzing-lsp-linux-x86_64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-lsp-linux-x86_64) |

### waltzing-mcp (MCP Server)

Model Context Protocol server for AI assistants like Claude Code.

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [waltzing-mcp-darwin-aarch64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-mcp-darwin-aarch64) |
| Linux (x86_64) | [waltzing-mcp-linux-x86_64](https://github.com/awesomike/waltzing/releases/latest/download/waltzing-mcp-linux-x86_64) |

## Features

- **Composable Components** - JSX-like function tags with default parameters
- **Type-Safe** - Templates compile to Rust code, errors caught at build time
- **AI-Powered** - MCP server for Claude and other AI assistants
- **Editor Integration** - LSP server with Zed and VS Code extensions
- **Streaming** - Both buffered and streaming render modes
- **Embedded Languages** - First-class JSON, JavaScript, and CSS blocks

## Documentation

- **Website**: [waltzing.awesomike.com](https://waltzing.awesomike.com)
- **Getting Started**: [waltzing.awesomike.com/docs/getting-started](https://waltzing.awesomike.com/docs/getting-started)
- **LLM Guide**: [waltzing.awesomike.com/raw/llms.txt](https://waltzing.awesomike.com/raw/llms.txt)

## Usage

### CLI Compiler

```bash
waltzing -i templates -o out
waltzing -i templates -o out --streaming --with-axum
```

### Multiple Input Directories

```bash
waltzing -i src/templates -i vendor=pkg/templates -o out
```

### Build Integration (build.rs)

```rust
use std::path::PathBuf;
use waltzing::TemplateConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TemplateConfig {
        templates_dir: PathBuf::from("templates"),
        out_dir: PathBuf::from(std::env::var("OUT_DIR")?),
    };
    waltzing::compile_templates(config)?;
    Ok(())
}
```

## License

MIT
