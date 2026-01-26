# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the **public GitHub repository** for Waltzing editor tooling. It contains only the tree-sitter grammar, editor extensions, and waltzing-ui component library. The main Waltzing compiler source is in a separate private repository.

## Repository Structure

- **tree-sitter/** - Tree-sitter grammar for Waltzing syntax (JavaScript-based grammar definition)
- **extensions/zed/** - Zed editor extension with LSP integration (Rust/WASM)
- **libraries/waltzing-ui/** - shadcn-style component library for Waltzing templates

## Build Commands

### Tree-sitter Grammar

```bash
cd tree-sitter
npm install
npx tree-sitter generate    # Generate parser from grammar.js
npx tree-sitter test        # Run grammar tests
npx tree-sitter build-wasm  # Build WASM for web editors
```

### Zed Extension

```bash
cd extensions/zed
cargo build --release --target wasm32-wasi
```

To install as dev extension in Zed: Command palette → "zed: install dev extension" → select the `extensions/zed` directory.

## GitHub Releases

Binary releases are published from this repository. The binaries are built in the private repo and uploaded here:

```bash
VERSION=0.2.27  # Set to current version

# Create release
gh release create "v${VERSION}" --title "v${VERSION}" --notes "Release ${VERSION}"

# Copy binaries with platform suffix
mkdir -p /tmp/waltzing-release
for platform in darwin-aarch64 darwin-x86_64; do
  for bin in waltzing waltzing-lsp waltzing-mcp; do
    cp ~/awesomike/tickets/waltzing/target/release-builds/v${VERSION}/${platform}/${bin} \
       /tmp/waltzing-release/${bin}-${platform}
  done
done

# Upload assets and mark as latest
gh release upload "v${VERSION}" /tmp/waltzing-release/*
gh release edit "v${VERSION}" --latest
```

## Architecture

### Tree-sitter Grammar (`tree-sitter/grammar.js`)

Defines the Waltzing template syntax including:
- Template directives: `@use`, `@import`, `@struct`, `@enum`, `@fn`
- Control flow: `@if`, `@for`, `@match`, `@let`
- Special keywords: `@Out`, `@out`, `@render(T1, T2, ...)`
- HTML integration with template function tags: `<@function_name>...</@>`

Reserved variable names (`__wtz_target`, `out`) are enforced by the compiler, not the grammar.

### Zed Extension (`extensions/zed/src/lib.rs`)

WASM extension that:
- Implements `zed_extension_api::Extension` trait
- Locates `waltzing-lsp` binary in common paths (~/.cargo/bin, ~/.local/bin, /usr/local/bin, etc.)
- Supports custom binary path via Zed settings

### waltzing-ui Library (shadcn/ui for Waltzing)

A shadcn/ui-style component library providing copy-paste or import-based components.

**Dependencies:**
- Tailwind CSS with CSS variables for theming
- Alpine.js with `@alpinejs/focus` and `@alpinejs/collapse` plugins

**Components:** button, card, dialog, dropdown, input, select, table, tabs
**Layouts:** base (with CDN setup), sidebar
**Utilities:** `cn()`, `class_if()`, `class_toggle()`

**Usage pattern:**
```waltzing
@import ui/components/button.wtz as button

<@button::primary>Click me</@>

@* Or with full control *@
<@button::apply
    variant=button::Variant::Destructive
    size=button::Size::Lg
    disabled=false
    class=None
>
    Delete
</@>
```

**Key patterns:**
- Components use `@()` render callbacks for content slots
- Variants defined with `@enum` (e.g., `Variant::Primary`, `Size::Lg`)
- `cn()` utility concatenates class names (like shadcn's `cn()`)
- Theme uses CSS variables (same names as shadcn: `--primary`, `--background`, etc.)

Component dependencies are defined in `waltzing-ui.toml`.

## File Types

Waltzing templates use these extensions:
- `.wtz` - Standard template files
- `.html.wtz`, `.css.wtz`, `.js.wtz` - Content-specific templates

## Agent & Skills

For Waltzing template expertise, see:
- `.claude/agents/waltzing-expert.md` - Complete syntax reference and best practices
- `.claude/skills.md` - Available skills for template writing and HTML conversion

## Waltzing MCP Tools

If `waltzing-mcp` is configured, these tools are available:

| Tool | Description |
|------|-------------|
| `validate` | Validate template syntax, returns errors with line/column |
| `syntax_help` | Get help for constructs (`if`, `for`, `fn`, `let`, `json`, `js`) |
| `list_constructs` | List constructs by category (`control-flow`, `definitions`, `expressions`, `embedded`) |
| `get_grammar` | Get documentation (`ebnf`, `reference`, `examples`) |
| `get_starter_project` | Scaffold a new Waltzing project |
| `inspect_template` | Parse template to extract functions, structs, enums, imports |

**Workflow for using components:**
1. Read the `.wtz` file containing the component
2. Call `inspect_template` with the file content
3. Use the returned signature to call the component with correct parameters

## Waltzing Syntax Quick Reference

```waltzing
@use crate::models::User              @* Import Rust types *@
@import "layouts/base.wtz" as layout  @* Import other templates *@

@fn apply(title: String, users: Vec<User>) {
    <@layout title=@title>
        <h1>@title</h1>
        @for user in users {
            <p>@user.name</p>
        }
    </@>
}
```

Key syntax points:
- `@variable` for output (auto-escaped), `@(expr)` for complex expressions
- `@let x = (a + b)` - operators need parentheses
- `<@component attr=@value>content</@>` - function tags for components
- `@* comment *@` - template comments (asterisks must match)
- `@render(T)` type and `@out` reference for higher-order functions

## CRITICAL: Attribute Syntax (Common Mistakes)

**#1 Mistake: Quotes around dynamic attribute values**

When an attribute value is a Waltzing expression, do NOT wrap it in quotes:

```waltzing
@* ❌ WRONG - quotes make @ a literal character, NOT evaluated *@
<div class="@container_cls" id="@my_id" />
<button class="@cn(&[base, extra])" />

@* ✅ CORRECT - no quotes, expression is evaluated *@
<div class=@container_cls id=@my_id />
<button class=@cn(&[base, extra]) />
```

**Rule:**
- `attr="literal"` → literal string value
- `attr=@expression` → evaluated expression (NO QUOTES!)
- `attr="prefix @var suffix"` → string interpolation (quotes OK for mixed content)

**#2 Mistake: x-data with variable interpolation**

When Alpine.js `x-data` contains Waltzing variables, use the embedded JSON format:

```waltzing
@* ❌ WRONG - variables inside quoted x-data are not evaluated *@
<div x-data="{ count: @initial, open: @is_open }">

@* ✅ CORRECT - use embedded JSON format *@
<div x-data=@```json { count: @initial, open: @is_open } ```@>
```

**#3 Mistake: Missing parentheses in @let**

```waltzing
@* ❌ WRONG *@
@let sum = a + b
@let valid = x > 0

@* ✅ CORRECT - operators need parentheses *@
@let sum = (a + b)
@let valid = (x > 0)
```

### Callback Syntax

When passing render callbacks to components:

- **`@() { ... }`** - Correct syntax for inline callbacks
  ```waltzing
  <@dialog::apply trigger=@() { <button>Open</button> } />
  @let my_callback = @() { <div>content</div> }
  ```

- **`@{ ... }`** - NOT valid for inline callbacks (causes parse errors)

Always use `@() { ... }` when passing callbacks as parameters to function tags.
