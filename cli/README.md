# Waltzing CLI

A shadcn/ui-style CLI for initializing Waltzing projects and adding components from the waltzing-ui library.

## Installation

The `waltzing` CLI is bundled with the main Waltzing binary. Install it via the [releases page](https://github.com/awesomike/waltzing-runtime/releases) or:

```bash
# macOS (Apple Silicon)
curl -fsSL https://github.com/awesomike/waltzing-runtime/releases/latest/download/waltzing-darwin-aarch64 -o ~/.local/bin/waltzing
chmod +x ~/.local/bin/waltzing

# macOS (Intel)
curl -fsSL https://github.com/awesomike/waltzing-runtime/releases/latest/download/waltzing-darwin-x86_64 -o ~/.local/bin/waltzing
chmod +x ~/.local/bin/waltzing
```

## Commands

### `waltzing init`

Initialize a new Waltzing project with interactive prompts.

```bash
waltzing init
```

This will:

1. Ask for your project name
2. Let you choose a framework (Axum, Actix, or Rocket)
3. Select whether to use TypeScript for build tooling
4. Configure Tailwind CSS and Alpine.js
5. Generate the project structure

**Options:**

| Flag | Description |
|------|-------------|
| `--name <name>` | Project name (skips prompt) |
| `--framework <fw>` | Framework: `axum`, `actix`, or `rocket` |
| `--typescript` | Include TypeScript build tooling |
| `--no-typescript` | Skip TypeScript (default) |
| `--yes` | Accept all defaults |

**Example:**

```bash
# Interactive
waltzing init

# Non-interactive with all options
waltzing init --name my-app --framework axum --no-typescript --yes
```

**Generated structure:**

```
my-app/
├── Cargo.toml
├── waltzing.toml
├── src/
│   └── main.rs
├── templates/
│   ├── base.wtz
│   └── index.wtz
├── ui/
│   └── lib/
│       └── utils.wtz
├── static/
│   └── .gitkeep
└── tailwind.config.js
```

---

### `waltzing add <component>`

Add a component from the waltzing-ui library to your project.

```bash
waltzing add button
```

This will:

1. Fetch the component from the waltzing-ui registry
2. Install it to your `ui/components/` directory
3. Automatically install any dependencies (e.g., `utils.wtz`)
4. Update your `waltzing.toml` with the new component

**Options:**

| Flag | Description |
|------|-------------|
| `--all` | Install all available components |
| `--path <path>` | Custom installation path |
| `--overwrite` | Overwrite existing components |
| `--dry-run` | Show what would be installed |

**Examples:**

```bash
# Add a single component
waltzing add button

# Add multiple components
waltzing add button card dialog

# Add with dependencies shown
waltzing add alert-dialog
# Installing: alert-dialog
# Dependencies: dialog, button, lib/utils
# Created: ui/components/alert-dialog.wtz

# Add a layout
waltzing add sidebar

# Preview without installing
waltzing add button --dry-run
```

**Available components:**

Run `waltzing add --list` to see all available components, or visit the [component showcase](https://waltzing.dev/components).

Components include:
- **Inputs:** button, input, textarea, checkbox, switch, select, radio-group
- **Display:** card, badge, avatar, table, tabs, accordion
- **Feedback:** alert, toast, dialog, progress, spinner
- **Navigation:** breadcrumb, pagination, navigation-menu
- **Overlays:** popover, dropdown, tooltip, sheet, drawer

---

### `waltzing add <block>`

Add pre-built blocks (larger compositions of components) to your project.

```bash
waltzing add contact-form
```

Blocks are installed to `ui/blocks/` and include all required component dependencies.

**Available blocks:**

| Block | Description | Dependencies |
|-------|-------------|--------------|
| `contact-form` | Contact form with validation | button, input, textarea, label |
| `profile-form` | User profile editor | avatar, button, card, field, form, input, input-group, separator, textarea |
| `confirm-dialog` | Generic confirmation dialog | dialog, button |
| `delete-dialog` | Destructive action confirmation | dialog |
| `share-dialog` | Share with link copy | dialog, button, input |

**Example:**

```bash
waltzing add profile-form
# Installing: profile-form
# Dependencies: avatar, button, card, field, form, input, input-group, separator, textarea, lib/utils
# Created: ui/blocks/forms/profile.wtz
```

---

### `waltzing theme <preset>`

Switch to a different theme preset. Theme presets change the CSS variables used by components.

```bash
waltzing theme zinc
```

**Available presets:**

| Preset | Description |
|--------|-------------|
| `default` | Default blue/slate theme (shadcn default) |
| `zinc` | Neutral zinc grays |
| `slate` | Cool slate grays |
| `stone` | Warm stone grays |
| `gray` | Pure gray tones |
| `neutral` | Balanced neutral |
| `red` | Red primary accent |
| `rose` | Rose/pink primary |
| `orange` | Orange primary |
| `green` | Green primary |
| `blue` | Blue primary (default) |
| `yellow` | Yellow primary |
| `violet` | Violet/purple primary |

**Options:**

| Flag | Description |
|------|-------------|
| `--list` | Show all available presets |
| `--preview` | Print CSS variables without applying |

**Example:**

```bash
# Switch to zinc theme
waltzing theme zinc

# Preview a theme
waltzing theme rose --preview

# List all themes
waltzing theme --list
```

The theme command modifies the CSS variables in your `ui/lib/theme.wtz` or base layout file.

---

## Configuration

### waltzing.toml

The project configuration file created by `waltzing init`:

```toml
[project]
name = "my-app"
version = "0.1.0"

[paths]
templates = "templates"
ui = "ui"
static = "static"
output = "target/waltzing"

[tailwind]
config = "tailwind.config.js"
input = "static/input.css"
output = "static/output.css"

[alpine]
plugins = ["@alpinejs/focus", "@alpinejs/collapse"]

[components]
# Components are added here by `waltzing add`
button = { version = "0.1.0" }
card = { version = "0.1.0" }

[theme]
preset = "default"
radius = "0.5rem"
```

### Custom component paths

You can customize where components are installed:

```toml
[paths]
ui = "src/templates/ui"  # Components go to src/templates/ui/components/
```

---

## Usage with Templates

After adding components, import and use them in your templates:

```waltzing
@import ui/components/button.wtz as button
@import ui/components/card.wtz as card

@fn page() {
    <@card::apply>
        <@card::header>
            <@card::title>Welcome</@>
        </@>
        <@card::content>
            <p>Hello from Waltzing!</p>
            <@button::primary>Get Started</@>
        </@>
    </@>
}
```

---

## Updating Components

To update components to the latest version:

```bash
# Update a specific component
waltzing add button --overwrite

# Update all components
waltzing add --all --overwrite
```

---

## Framework-specific Setup

### Axum

The Axum starter includes:
- Template rendering with `waltzing::render()`
- Static file serving from `/static`
- Hot reload in development

```rust
use axum::{Router, routing::get};
use waltzing::Template;

#[derive(Template)]
#[template(path = "index.wtz")]
struct IndexTemplate {
    title: String,
}
```

### Actix

```rust
use actix_web::{web, App, HttpServer};
use waltzing::Template;

#[derive(Template)]
#[template(path = "index.wtz")]
struct IndexTemplate {
    title: String,
}
```

### Rocket

```rust
use rocket::get;
use waltzing::Template;

#[derive(Template)]
#[template(path = "index.wtz")]
struct IndexTemplate {
    title: String,
}
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WALTZING_UI_REGISTRY` | Custom component registry URL | `https://waltzing.dev/registry` |
| `WALTZING_CONFIG` | Path to waltzing.toml | `./waltzing.toml` |

---

## See Also

- [Waltzing Documentation](https://waltzing.dev/docs)
- [Component Library](https://waltzing.dev/components)
- [Template Syntax Reference](https://waltzing.dev/syntax)
