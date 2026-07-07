<div align="center">

# Waltzing

### The compile-time HTML template engine for Rust

Write components in `.wtz` files. Waltzing turns them into **type-safe Rust functions** at build time — escaped by default, checked by the compiler, with zero runtime template parsing.

[![Release](https://img.shields.io/github/v/release/awesomike/waltzing?color=8b5cf6)](https://github.com/awesomike/waltzing/releases)
[![License](https://img.shields.io/badge/license-MIT-14b8a6)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange)](https://www.rust-lang.org)

[Website](https://waltzing.awesomike.com) ·
[Docs](https://waltzing.awesomike.com/docs) ·
[Why Waltzing?](https://waltzing.awesomike.com/docs/comparison) ·
[Editor Support](#editor-support)

</div>

---

## Why Waltzing?

Most template engines for Rust parse strings and resolve variables **at runtime** — which means a typo in a template is a `500` in production, your view data is stringly-typed, and every request pays a parsing cost.

Waltzing does the work at **build time**. A `.wtz` template compiles to an ordinary Rust function: parameters are real types, expressions are real Rust, output is auto-escaped, and a mistake fails `cargo build` — not your users' requests.

|                          | Runtime engines (Tera, Handlebars, …) | **Waltzing**                          |
| ------------------------ | ------------------------------------- | ------------------------------------- |
| When errors surface      | In production, at render time         | At `cargo build`                      |
| View data                | Stringly-typed context                | Real Rust types                       |
| Per-request cost         | Parse + interpret every time          | Compiled once, native at runtime      |
| HTML escaping            | Opt-in, easy to forget                | On by default, everywhere             |
| Refactoring              | No compiler help                      | Rename a field → compiler finds every use |

## A quick taste

```waltzing
@* templates/components/card.wtz *@
@fn card(title: &str, body: Content) {
    <article class="card">
        <h2>@title</h2>
        @body
    </article>
}
```

```waltzing
@* templates/pages/home.wtz *@
@import /components/card.wtz as card
@use crate::models::User

@fn page(user: &User, unread: usize) {
    <@card::apply title="Welcome back">
        <p>Signed in as @user.email</p>
        <p>You have @unread @pluralize(unread, "message")</p>
    </@>

    @if user.is_admin {
        <a href=@safe_url(&user.admin_url)>Admin console</a>
    }
}
```

`<`, `>` and `&` are escaped automatically; `@user.email` is type-checked against your real `User` struct; `safe_url` blocks `javascript:` URIs. Rename `email` and the Rust compiler tells you every template that needs updating.

## Install

```bash
# macOS / Linux
curl -fsSL https://waltzing.awesomike.com/install | bash
```

```powershell
# Windows (PowerShell)
irm https://waltzing.awesomike.com/install.ps1 | iex
```

This installs the `waltzing` compiler plus the `waltzing-lsp` and `waltzing-mcp` binaries. Prebuilt for macOS (Apple Silicon & Intel) and Linux (x86-64 & aarch64) — see [Releases](https://github.com/awesomike/waltzing/releases).

## Use it in a project

**1. Put templates in a `templates/` directory** (the example above).

**2. Compile them in `build.rs`** — Waltzing generates one Rust module per template:

```rust
// build.rs
use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("templates");
    // find `waltzing` on PATH (falls back to ~/.local/bin)
    let status = Command::new("waltzing")
        .args(["-i", "templates", "--with-axum", "-o", out.to_str().unwrap()])
        .status()
        .expect("run waltzing");
    assert!(status.success(), "template compilation failed");
    println!("cargo:rerun-if-changed=templates");
}
```

**3. Include the generated code and render** — a template returns `Content`, which is an axum `IntoResponse` when built with `--with-axum`:

```rust
mod templates {
    include!(concat!(env!("OUT_DIR"), "/templates/mod.rs"));
}

async fn home(State(db): State<Db>) -> impl IntoResponse {
    let user = db.current_user().await?;
    templates::pages::home::page(&user, db.unread(&user).await?)
}
```

That's it — no runtime dependency on the template engine, no parsing at request time. The rendered HTML is produced by plain Rust string building.

## Features

- **Type-safe components.** Templates are functions. Import them as modules and call them with JSX-like function tags (`<@card::apply .../>`), with default parameters and content slots.
- **Escaped by default.** Text interpolation is HTML-escaped automatically; `safe_url`, `safe_attr`, and `json` cover the sharp edges. XSS-safe without thinking about it.
- **Real control flow.** `@if`, `@for`, `@match` over ordinary Rust — including guards, patterns, and `else`.
- **Batteries-included helpers.** `currency`, `format_date`, `pluralize`, `truncate`, `cn`, `number`, `percent`, and more — tree-shaken, so you only ship what you use.
- **Embedded languages.** First-class JavaScript, CSS, and JSON blocks with interpolation. Pairs beautifully with **htmx** and **Alpine.js**.
- **Buffered, streaming, and async.** Render to a `String`, stream chunks for faster time-to-first-byte, or write asynchronously to any sink.
- **Compile-time i18n.** `@translate("key", name = expr)` validated against per-locale catalogs at build time.
- **Great errors.** rustc errors are source-mapped back to the exact `.wtz` line; a formatter and auto-fix keep templates tidy.

Full guide and syntax reference at **[waltzing.awesomike.com/docs](https://waltzing.awesomike.com/docs)**.

## Editor support

- **Zed** — published on the Zed extension marketplace (`waltzing`).
- **VS Code** — extension in [`extensions/vscode`](extensions/vscode) with syntax highlighting, LSP, and format-on-save.
- **Any LSP editor** — the `waltzing-lsp` binary provides autocomplete and diagnostics.
- **Tree-sitter grammar** — in [`tree-sitter/`](tree-sitter), powering highlighting and structural editing.

## AI-native

Waltzing ships an **MCP server** (`waltzing-mcp`) that gives Claude Code and other assistants tools to read, write, and validate templates, plus an LLM guide so they generate correct syntax. See [waltzing.awesomike.com/download](https://waltzing.awesomike.com/download).

## What's in this repository

This is the public home for Waltzing's tooling and shared assets:

- **[`libraries/waltzing-ui`](libraries/waltzing-ui)** — a shadcn-inspired component library of ready-made Waltzing components (Tailwind + Alpine.js).
- **[`tree-sitter/`](tree-sitter)** — the tree-sitter grammar.
- **[`extensions/`](extensions)** — editor extensions (Zed, VS Code).
- **`src/`** — the Axum showcase server that browses the component libraries.
- **`releases/`** — packaged binaries and the install manifest.

The `waltzing-ui` library and the showcase are compiled by [`build.rs`](build.rs), which discovers each library's manifest and invokes the `waltzing` CLI. The build searches `PATH`, then `~/.local/bin`; set `WALTZING_BIN` to force a specific compiler.

```bash
# build & test the libraries and showcase
cargo test --locked

# run the showcase locally
cargo run
```

## License

MIT © awesomike. See [LICENSE](LICENSE).
