# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Overview

Public GitHub repository for Waltzing editor tooling: tree-sitter grammar, editor extensions, and waltzing-ui component library. The main Waltzing compiler is in a separate private repository.

## Repository Structure

- **tree-sitter/** - Tree-sitter grammar for Waltzing syntax (JavaScript-based)
- **extensions/zed/** - Zed editor extension with LSP integration (Rust/WASM)
- **libraries/waltzing-ui/** - shadcn-style component library for Waltzing templates

## Build Commands

```bash
# Tree-sitter grammar
cd tree-sitter && npm install
npx tree-sitter generate    # Generate parser
npx tree-sitter test        # Run tests

# Zed Extension
cd extensions/zed && cargo build --release --target wasm32-wasi
# Install: Command palette → "zed: install dev extension" → select extensions/zed
```

## File Types

- `.wtz` - Standard template files
- `.html.wtz`, `.css.wtz`, `.js.wtz` - Content-specific templates

## waltzing-ui Library

shadcn/ui-style component library. Requires Tailwind CSS and Alpine.js with `@alpinejs/focus` and `@alpinejs/collapse` plugins.

```waltzing
@import ui/components/button.wtz as button

<@button::primary>Click me</@>

<@button::apply variant=button::Variant::Destructive size=button::Size::Lg>
    Delete
</@>
```

**Patterns:**
- Components use `@()` render callbacks for content slots
- Variants defined with `@enum` (e.g., `Variant::Primary`, `Size::Lg`)
- `cn()` concatenates class names (like shadcn's `cn()`)
- Theme uses CSS variables (`--primary`, `--background`, etc.)
- Dependencies defined in `waltzing-ui.toml`

## MCP Tools (waltzing-mcp)

| Tool | Description |
|------|-------------|
| `validate` | Validate syntax, returns errors with line/column |
| `syntax_help` | Help for constructs (`if`, `for`, `fn`, `let`, `json`, `js`) |
| `list_constructs` | List constructs by category |
| `inspect_template` | Parse template to extract functions, structs, enums, imports |
| `lint_template` | Check common mistakes before validation |

**Workflow for components:** Read `.wtz` file → `inspect_template` → use returned signature.

### Waltzing Templates (.wtz files)

#### IMPORTANT: Common Mistakes to Avoid

The `@` prefix **enters expression mode** from template mode. Once you're IN expression mode, don't use `@` again:

```waltzing
@* ❌ WRONG - no @ after = *@
@let x = @if cond { a } else { b }
@let arr = vec![@a, @b]

@* ✅ CORRECT - plain Rust after = *@
@let x = if cond { a } else { b }
@let arr = vec![a, b]

@* ❌ WRONG - quotes make it literal *@
<@component name="@user.name" />

@* ✅ CORRECT - no quotes for expressions *@
<@component name=@user.name />

@* ❌ WRONG - operators need parentheses *@
@let sum = a + b

@* ✅ CORRECT *@
@let sum = (a + b)
```

#### Quick Reference

```waltzing
@* Imports and definitions *@
@use crate::models::User
@import "layouts/base.wtz" as layout

@* Variable bindings - PARENTHESES REQUIRED for operators *@
@let name = user.name                      @* Simple: OK *@
@let total = (price * quantity)            @* Arithmetic: needs parens *@
@let valid = (age > 18 && active)          @* Boolean ops: needs parens *@

@* Output expressions *@
@user.name                                 @* Simple variable *@
@(items.len() + 1)                         @* Complex expr: use parens *@
@safe(html_content)                        @* Unescaped HTML *@
@&user.display_name                        @* Reference to avoid move *@

@* Control flow *@
@if condition { ... } else { ... }
@if let Some(x) = opt { ... }
@for item in items { ... }
@match value { Pattern => { ... } }
```

#### Spread Syntax and cn() Helper

```waltzing
@* cn() class helper - variadic syntax auto-wraps in array *@
@cn("btn", "primary", if active { "active" } else { "" })

@* Spread arrays into function arguments with ... *@
@let extras = ["hover", "focus"]
@cn("base", extras...)                     @* Spreads extras into cn() *@
@cn("a", x..., "b", y...)                  @* Multiple spreads, order preserved *@

<div class=@cn("btn", conditional_classes..., "primary") />
```

#### Control Flow in HTML Attributes

```waltzing
<button
    @if is_disabled { disabled }
    @if is_primary { class="btn-primary" }
>Submit</button>

@* Nested control flow requires @ prefix *@
<input @if let Some(x) = opt { @if x.active { checked } } />
```

#### Function Tags (Components)

```waltzing
<@button label="Submit" disabled=@is_submitting />

<@layout::page title="Dashboard">
    <div class="content">@page_content</div>
</@layout::page>

@* Shorthand closing *@
<@card title="Stats"><p>Content</p></@>
```

#### Render Callbacks

```waltzing
@* Define function with render callback parameter *@
@fn dropdown(header: String, content: @()) {
    <div class="header">@header</div>
    <div class="body">@content()</div>
}

@* Call with inline template *@
<@dropdown header="Menu" content=@() {
    <ul><li>Item 1</li><li>Item 2</li></ul>
}/>
```

#### Built-in Helpers (selection)

```waltzing
@* Text *@
@truncate(&text, 100)                      @* Truncate with "..." *@
@pluralize(count, "item")                  @* "1 item" or "5 items" *@

@* Numbers *@
@number(count)                             @* 1234567 -> "1,234,567" *@
@currency(1234.50, "USD")                  @* "$1,234.50" *@

@* HTML/CSS *@
@cn("btn", "primary", if active { "active" } else { "" })
@class_if(is_active, "active")
@json(&data)                               @* JSON encode for <script> *@
@safe_url(user_url)                        @* Validates URL is safe *@
```

## References

- `.claude/agents/waltzing-expert.md` - Complete syntax reference
- `.claude/skills.md` - Available skills for template writing
