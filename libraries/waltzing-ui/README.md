# waltzing-ui

`waltzing-ui` is a shadcn-inspired component library for
[Waltzing](https://github.com/awesomike/waltzing) templates. It is designed to
be copied into an application or imported as a template library.

## Status

This directory is the stable, compiling component set. Experimental templates
that do not compile with the current Waltzing parser should not live in this
directory, because Waltzing parses every `.wtz` file under an input root.

The runtime build validates:

- every `waltzing-ui.toml` path exists;
- every manifest dependency points at a known entry or `lib/*` helper;
- every registry entry points at a manifest entry with valid dependencies;
- the stable library surface keeps real breadth across components, layouts, and
  blocks;
- block and dialog templates keep basic accessibility markers such as labels,
  alerts, dialog roles, and accessible names;
- the whole directory compiles with the Waltzing CLI.

Run the same gate directly:

```bash
waltzing -i libraries/waltzing-ui --with-axum -o /tmp/waltzing-ui-check
```

## Installation

Import the library with an alias:

```bash
waltzing -i templates -i path/to/waltzing-ui=ui -o out
```

Then import components from templates:

```waltzing
@import ui/components/button.wtz as button
@import ui/layouts/base.wtz as base

@fn apply() {
    <@base::apply title="My Page" description=None head=None>
        <@button::primary>Save</@button::primary>
    </@base::apply>
}
```

You can also copy individual components and their dependencies into your app.
Use `waltzing-ui.toml` as the dependency map.

## Included Components

Core UI:

- `accordion`, `alert`, `alert-dialog`, `aspect-ratio`, `avatar`, `badge`,
  `breadcrumb`, `button`, `button-group`, `card`, `collapsible`, `dialog`,
  `dialogs-dialog`, `dialogs-sheet`, `dialogs-workspace`, `drawer`,
  `dropdown`, `hover-card`, `kbd`, `pagination`, `popover`, `progress`,
  `scroll-area`, `separator`, `sheet`, `skeleton`, `spinner`, `tabs`, `toast`,
  `tooltip`, `typography`

Forms and inputs:

- `checkbox`, `date-picker`, `field`, `file-upload`, `form`, `input`, `label`,
  `minmax-input`, `numeric-field`, `radio-group`, `select`, `slider`, `switch`,
  `textarea`, `toggle`, `toggle-group`, `validation-errors`

Navigation and layout:

- `command`, `context-menu`, `table`, `layouts/base`, `layouts/dashboard`,
  `layouts/sidebar`

Advanced inputs:

- `multi-select`

## Included Blocks

The stable block set covers common application starting points:

- `login-card` and `signup-card` for accessible auth forms;
- `contact-card` for labeled contact forms with validation message slots;
- `stats-grid` for dashboard KPI summaries.

Blocks are intentionally held to the same parser and registry checks as
components. Experimental blocks should land outside `libraries/waltzing-ui`
until they compile with the current Waltzing parser.

## Runtime Dependencies

Most interactive components use Alpine.js. Components that trap focus or
collapse content require Alpine plugins:

- `@alpinejs/focus`
- `@alpinejs/collapse`

The default base layout includes CDN scripts for convenience. Production apps
should usually bundle Tailwind and Alpine locally, pin versions, and apply a
Content Security Policy that matches the deployment.

## Security Boundaries

Waltzing escapes ordinary text and attribute values by default. Components that
accept JavaScript expressions or raw HTML must be treated differently:

- Props named like `on_*`, `*_handler`, `open_var`, `alpine_model`, or `model`
  are trusted Alpine/JavaScript expressions, not user content.
- Icon slots and other `@safe(...)` HTML are trusted markup.
- Dynamic `href` and `src` values supplied by applications should be normalized
  before passing into components. Prefer Waltzing URL escaping helpers when the
  value can contain user input.
- Select-like components quote user-visible strings before embedding them in
  Alpine state. Keep this pattern when adding new JS-backed components.

## Component Guidelines

When adding or changing components:

1. Keep every `.wtz` file in this directory parseable.
2. Add the component to both `waltzing-ui.toml` and `registry.json`.
3. Keep dependency names consistent between the manifest and registry.
4. Use `@use waltzing::escape::attr` before embedding strings into generated
   JavaScript literals.
5. Prefer computed `@let` class strings over `@...` inside quoted attributes.
6. Run the direct Waltzing compile command and `cargo test --locked`.

## Versioning

`waltzing-ui.toml` and `registry.json` should carry the same version. A version
bump means the component API, component list, or dependency metadata changed.

## License

MIT
