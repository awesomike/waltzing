# waltzing-ui

A shadcn-inspired component library for [Waltzing](https://github.com/awesomike/waltzing) templates.

## Features

- **Copy or Import** - Use components via copy-paste (shadcn style) or import from directory
- **Alpine.js Integration** - Interactive components with focus traps, transitions, and state
- **Tailwind CSS** - Styled with Tailwind and CSS variables for theming
- **Type-Safe** - Full Rust type safety with render callbacks (`@()` pattern)
- **Light/Dark Mode** - Built-in theme support via CSS variables

## Installation

### Option 1: Import from Directory

Add the library as an import alias in your build:

```bash
waltzing -i templates -i path/to/waltzing-ui=ui -o out
```

Then import components in your templates:

```waltzing
@import ui/components/button.wtz as button
@import ui/layouts/base.wtz as base

@fn apply() {
    <@base::apply title="My Page" description=None head=None>
        <@button::apply variant=button::Variant::Primary size=button::Size::Md disabled=false class=None>
            Click me
        </@>
    </@>
}
```

### Option 2: Copy Components (shadcn style)

Copy individual components into your project:

```bash
# Copy button component and its dependencies
cp waltzing-ui/components/button.wtz your-project/templates/components/
cp waltzing-ui/lib/utils.wtz your-project/templates/lib/
```

## Components

| Component | Description |
|-----------|-------------|
| `button` | Clickable button with multiple variants and sizes |
| `card` | Container with optional header and footer slots |
| `input` | Text input with optional label and error state |
| `select` | Dropdown select input with custom styling |
| `dialog` | Modal dialog with backdrop, focus trap, and escape handling |
| `dropdown` | Dropdown menu with trigger and items |
| `table` | Responsive data table with header and row rendering |
| `tabs` | Tabbed interface with Alpine.js state management |

## Layouts

| Layout | Description |
|--------|-------------|
| `base` | HTML document structure with head and body |
| `sidebar` | Dashboard layout with collapsible sidebar |

## Dependencies

### Required

- [Tailwind CSS](https://tailwindcss.com/) with `@tailwindcss/forms` plugin
- [Alpine.js](https://alpinejs.dev/) with plugins:
  - `@alpinejs/focus` - Focus trap for dialogs
  - `@alpinejs/collapse` - Collapse animations

### CDN Setup (included in base layout)

The `layouts/base.wtz` includes CDN links for Tailwind and Alpine.js. For production, consider using local builds.

## Theming

Components use CSS variables for theming. Override in your CSS:

```css
:root {
  --primary: 221.2 83.2% 53.3%;
  --primary-foreground: 210 40% 98%;
  /* ... see theme/default.css for all variables */
}

.dark {
  --primary: 217.2 91.2% 59.8%;
  /* ... dark mode overrides */
}
```

## Usage Examples

### Button with Variants

```waltzing
@import ui/components/button.wtz as button

<@button::primary disabled=false class=None>
    Primary Button
</@>

<@button::secondary disabled=false class=None>
    Secondary Button
</@>

<@button::destructive disabled=false class=None>
    Delete
</@>
```

### Card with Slots

```waltzing
@import ui/components/card.wtz as card

<@card::apply class=None>
    <@card::header>
        <h2>Card Title</h2>
    </@>
    <p>Card content goes here.</p>
    <@card::footer>
        <button>Action</button>
    </@>
</@>
```

### Dialog

```waltzing
@import ui/components/dialog.wtz as dialog
@import ui/components/button.wtz as button

<@dialog::apply
    title="Confirm Action"
    description=Some("Are you sure you want to proceed?")
    trigger=@() { <@button::primary disabled=false class=None>Open Dialog</@> }
    footer=Some(@() {
        <@button::secondary disabled=false class=None>Cancel</@>
        <@button::primary disabled=false class=None>Confirm</@>
    })
>
    <p>Dialog content here.</p>
</@>
```

### Data Table

```waltzing
@import ui/components/table.wtz as table

@struct User {
    name: String,
    email: String,
}

<@table::apply
    items=@users
    header=@() {
        <@table::head>Name</@>
        <@table::head>Email</@>
    }
    row=@(user: &User) {
        <@table::cell>@user.name</@>
        <@table::cell>@user.email</@>
    }
    empty=Some(@() { <p>No users found.</p> })
    class=None
/>
```

### Tabs

```waltzing
@import ui/components/tabs.wtz as tabs

@let tab_list = vec![
    tabs::Tab { id: "overview".to_string(), label: "Overview".to_string() },
    tabs::Tab { id: "settings".to_string(), label: "Settings".to_string() },
];

<@tabs::apply tabs=@tab_list.as_slice() default_tab="overview" class=None content=@(tab_id: &str) {
    @match tab_id {
        "overview" => { <p>Overview content</p> }
        "settings" => { <p>Settings content</p> }
        _ => {}
    }
}/>
```

## Customization

### Local Overrides

Create a local version of any component to customize:

```
templates/
  components/
    button.wtz  # Your customized button
  lib/
    utils.wtz   # Your utilities
```

### Wrapper Pattern

Wrap components to add project-specific defaults:

```waltzing
@import ui/components/button.wtz as ui_button

@fn primary_button(content: @()) {
    <@ui_button::apply
        variant=ui_button::Variant::Primary
        size=ui_button::Size::Md
        disabled=false
        class=Some("my-custom-class")
        content=@content
    />
}
```

## License

MIT
