//! Waltzing Showcase Server
//!
//! A web server that showcases all discovered Waltzing template libraries.
//! Libraries are auto-discovered from the `libraries/` directory at compile time.

use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod generated;

use generated::LIBRARIES;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "waltzing_showcase=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build the router
    let app = Router::new()
        .route("/", get(index))
        .route("/library/{id}", get(library_showcase))
        .route("/library/{id}/component/{component}", get(component_showcase))
        .nest_service("/static", ServeDir::new("static"));

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Showcase server running at http://127.0.0.1:3000");
    tracing::info!("Discovered {} libraries", LIBRARIES.len());

    for lib in LIBRARIES {
        tracing::info!(
            "  - {} v{} ({} components)",
            lib.name,
            lib.version,
            lib.component_count
        );
    }

    axum::serve(listener, app).await.expect("Server error");
}

fn head_common() -> &'static str {
    r#"
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        border: 'hsl(var(--border))',
                        input: 'hsl(var(--input))',
                        ring: 'hsl(var(--ring))',
                        background: 'hsl(var(--background))',
                        foreground: 'hsl(var(--foreground))',
                        primary: {
                            DEFAULT: 'hsl(var(--primary))',
                            foreground: 'hsl(var(--primary-foreground))',
                        },
                        secondary: {
                            DEFAULT: 'hsl(var(--secondary))',
                            foreground: 'hsl(var(--secondary-foreground))',
                        },
                        destructive: {
                            DEFAULT: 'hsl(var(--destructive))',
                            foreground: 'hsl(var(--destructive-foreground))',
                        },
                        muted: {
                            DEFAULT: 'hsl(var(--muted))',
                            foreground: 'hsl(var(--muted-foreground))',
                        },
                        accent: {
                            DEFAULT: 'hsl(var(--accent))',
                            foreground: 'hsl(var(--accent-foreground))',
                        },
                        popover: {
                            DEFAULT: 'hsl(var(--popover))',
                            foreground: 'hsl(var(--popover-foreground))',
                        },
                        card: {
                            DEFAULT: 'hsl(var(--card))',
                            foreground: 'hsl(var(--card-foreground))',
                        },
                    },
                },
            },
        }
    </script>
    <style>
        :root {
            --background: 0 0% 100%;
            --foreground: 222.2 84% 4.9%;
            --card: 0 0% 100%;
            --card-foreground: 222.2 84% 4.9%;
            --popover: 0 0% 100%;
            --popover-foreground: 222.2 84% 4.9%;
            --primary: 222.2 47.4% 11.2%;
            --primary-foreground: 210 40% 98%;
            --secondary: 210 40% 96.1%;
            --secondary-foreground: 222.2 47.4% 11.2%;
            --muted: 210 40% 96.1%;
            --muted-foreground: 215.4 16.3% 46.9%;
            --accent: 210 40% 96.1%;
            --accent-foreground: 222.2 47.4% 11.2%;
            --destructive: 0 84.2% 60.2%;
            --destructive-foreground: 210 40% 98%;
            --border: 214.3 31.8% 91.4%;
            --input: 214.3 31.8% 91.4%;
            --ring: 222.2 84% 4.9%;
            --radius: 0.5rem;
        }

        .dark {
            --background: 222.2 84% 4.9%;
            --foreground: 210 40% 98%;
            --card: 222.2 84% 4.9%;
            --card-foreground: 210 40% 98%;
            --popover: 222.2 84% 4.9%;
            --popover-foreground: 210 40% 98%;
            --primary: 210 40% 98%;
            --primary-foreground: 222.2 47.4% 11.2%;
            --secondary: 217.2 32.6% 17.5%;
            --secondary-foreground: 210 40% 98%;
            --muted: 217.2 32.6% 17.5%;
            --muted-foreground: 215 20.2% 65.1%;
            --accent: 217.2 32.6% 17.5%;
            --accent-foreground: 210 40% 98%;
            --destructive: 0 62.8% 30.6%;
            --destructive-foreground: 210 40% 98%;
            --border: 217.2 32.6% 17.5%;
            --input: 217.2 32.6% 17.5%;
            --ring: 212.7 26.8% 83.9%;
        }

        body {
            background-color: hsl(var(--background));
            color: hsl(var(--foreground));
        }

        [x-cloak] {
            display: none !important;
        }

        .component-preview {
            background: repeating-linear-gradient(
                45deg,
                hsl(var(--muted) / 0.3),
                hsl(var(--muted) / 0.3) 10px,
                transparent 10px,
                transparent 20px
            );
        }

        /* Scrollbar styling */
        * {
            scrollbar-width: thin;
            scrollbar-color: hsl(var(--border)) transparent;
        }

        *::-webkit-scrollbar {
            width: 8px;
            height: 8px;
        }

        *::-webkit-scrollbar-track {
            background: transparent;
        }

        *::-webkit-scrollbar-thumb {
            background-color: hsl(var(--border));
            border-radius: 4px;
        }

        *::-webkit-scrollbar-thumb:hover {
            background-color: hsl(var(--muted-foreground));
        }
    </style>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/focus@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/collapse@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
"#
}

fn theme_toggle() -> &'static str {
    r#"
<button
    @click="dark = !dark"
    class="p-2 rounded-md border border-border hover:bg-accent transition-colors"
    :aria-label="dark ? 'Switch to light mode' : 'Switch to dark mode'"
>
    <svg x-show="dark" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="4"></circle>
        <path d="M12 2v2"></path>
        <path d="M12 20v2"></path>
        <path d="m4.93 4.93 1.41 1.41"></path>
        <path d="m17.66 17.66 1.41 1.41"></path>
        <path d="M2 12h2"></path>
        <path d="M20 12h2"></path>
        <path d="m6.34 17.66-1.41 1.41"></path>
        <path d="m19.07 4.93-1.41 1.41"></path>
    </svg>
    <svg x-show="!dark" x-cloak xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"></path>
    </svg>
</button>
"#
}

/// Index page showing all discovered libraries
async fn index() -> impl IntoResponse {
    let libraries_html: String = LIBRARIES
        .iter()
        .map(|lib| {
            format!(
                r#"
                <a href="/library/{id}" class="block p-6 bg-card rounded-lg border border-border hover:border-primary transition-colors">
                    <div class="flex items-center justify-between mb-2">
                        <h2 class="text-xl font-semibold">{name}</h2>
                        <span class="text-sm text-muted-foreground">v{version}</span>
                    </div>
                    <p class="text-muted-foreground mb-4">{description}</p>
                    <div class="text-sm text-muted-foreground">
                        {count} components
                    </div>
                </a>
                "#,
                id = lib.id,
                name = lib.name,
                version = lib.version,
                description = lib.description,
                count = lib.component_count
            )
        })
        .collect();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="container mx-auto px-4 py-8 max-w-4xl">
        <header class="flex items-center justify-between mb-12">
            <div>
                <h1 class="text-4xl font-bold mb-2">Waltzing Showcase</h1>
                <p class="text-muted-foreground">Explore template libraries for the Waltzing engine</p>
            </div>
            {toggle}
        </header>

        <main class="grid gap-4">
            {libraries}
        </main>

        <footer class="mt-12 pt-8 border-t border-border text-center text-sm text-muted-foreground">
            <p>Powered by <a href="https://github.com/awesomike/waltzing" class="underline hover:text-foreground">Waltzing</a></p>
        </footer>
    </div>
</body>
</html>"#,
        head = head_common(),
        toggle = theme_toggle(),
        libraries = libraries_html
    );

    Html(html)
}

/// Library showcase page with all components
async fn library_showcase(Path(id): Path<String>) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let components = get_component_list(&id);
            let sidebar = generate_sidebar(&id, &components, None);
            let content = generate_all_components_preview(&id, &components);

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} - Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: true }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card overflow-y-auto transition-all duration-300"
        >
            <div class="p-4 border-b border-border">
                <a href="/" class="flex items-center gap-2 text-muted-foreground hover:text-foreground mb-4">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="m12 19-7-7 7-7"></path>
                        <path d="M19 12H5"></path>
                    </svg>
                    <span class="text-sm">Back</span>
                </a>
                <h1 class="text-lg font-semibold">{name}</h1>
                <p class="text-sm text-muted-foreground">v{version}</p>
            </div>
            {sidebar}
        </aside>

        <!-- Main content -->
        <main class="flex-1 overflow-y-auto">
            <header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div class="flex items-center gap-4">
                    <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="4" x2="20" y1="12" y2="12"></line>
                            <line x1="4" x2="20" y1="6" y2="6"></line>
                            <line x1="4" x2="20" y1="18" y2="18"></line>
                        </svg>
                    </button>
                    <h2 class="text-xl font-semibold">All Components</h2>
                </div>
                {toggle}
            </header>
            <div class="p-6">
                {content}
            </div>
        </main>
    </div>
</body>
</html>"#,
                name = lib.name,
                head = head_common(),
                version = lib.version,
                sidebar = sidebar,
                toggle = theme_toggle(),
                content = content
            );

            Html(html)
        }
        None => Html(not_found_page(&id)),
    }
}

/// Single component showcase page
async fn component_showcase(Path((id, component)): Path<(String, String)>) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let components = get_component_list(&id);
            let sidebar = generate_sidebar(&id, &components, Some(&component));
            let content = generate_component_detail(&component);

            let title = component
                .split('-')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - {lib_name} - Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: true }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card overflow-y-auto transition-all duration-300"
        >
            <div class="p-4 border-b border-border">
                <a href="/" class="flex items-center gap-2 text-muted-foreground hover:text-foreground mb-4">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="m12 19-7-7 7-7"></path>
                        <path d="M19 12H5"></path>
                    </svg>
                    <span class="text-sm">Back</span>
                </a>
                <h1 class="text-lg font-semibold">{lib_name}</h1>
                <p class="text-sm text-muted-foreground">v{version}</p>
            </div>
            {sidebar}
        </aside>

        <!-- Main content -->
        <main class="flex-1 overflow-y-auto">
            <header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div class="flex items-center gap-4">
                    <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="4" x2="20" y1="12" y2="12"></line>
                            <line x1="4" x2="20" y1="6" y2="6"></line>
                            <line x1="4" x2="20" y1="18" y2="18"></line>
                        </svg>
                    </button>
                    <h2 class="text-xl font-semibold">{title}</h2>
                </div>
                {toggle}
            </header>
            <div class="p-6 max-w-4xl">
                {content}
            </div>
        </main>
    </div>
</body>
</html>"#,
                title = title,
                lib_name = lib.name,
                head = head_common(),
                version = lib.version,
                sidebar = sidebar,
                toggle = theme_toggle(),
                content = content
            );

            Html(html)
        }
        None => Html(not_found_page(&id)),
    }
}

fn not_found_page(id: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Not Found</title></head>
<body style="font-family: system-ui; padding: 2rem;">
    <h1>Library not found: {}</h1>
    <a href="/">Back to home</a>
</body>
</html>"#,
        id
    )
}

fn get_component_list(library_id: &str) -> Vec<(&'static str, &'static str)> {
    match library_id {
        "waltzing-ui" => vec![
            ("accordion", "Accordion"),
            ("alert", "Alert"),
            ("alert-dialog", "Alert Dialog"),
            ("avatar", "Avatar"),
            ("badge", "Badge"),
            ("breadcrumb", "Breadcrumb"),
            ("button", "Button"),
            ("card", "Card"),
            ("checkbox", "Checkbox"),
            ("collapsible", "Collapsible"),
            ("combobox", "Combobox"),
            ("dialog", "Dialog"),
            ("dropdown", "Dropdown"),
            ("form", "Form"),
            ("input", "Input"),
            ("label", "Label"),
            ("popover", "Popover"),
            ("progress", "Progress"),
            ("radio-group", "Radio Group"),
            ("select", "Select"),
            ("separator", "Separator"),
            ("skeleton", "Skeleton"),
            ("slider", "Slider"),
            ("switch", "Switch"),
            ("table", "Table"),
            ("tabs", "Tabs"),
            ("textarea", "Textarea"),
            ("toggle", "Toggle"),
            ("tooltip", "Tooltip"),
        ],
        _ => vec![],
    }
}

fn generate_sidebar(
    library_id: &str,
    components: &[(&str, &str)],
    active: Option<&str>,
) -> String {
    let nav_items: String = components
        .iter()
        .map(|(id, name)| {
            let is_active = active == Some(*id);
            let active_class = if is_active {
                "bg-accent text-accent-foreground"
            } else {
                "text-muted-foreground hover:text-foreground hover:bg-accent/50"
            };
            format!(
                r#"<a href="/library/{lib}/component/{comp}" class="block px-4 py-2 text-sm rounded-md transition-colors {cls}">{name}</a>"#,
                lib = library_id,
                comp = id,
                cls = active_class,
                name = name
            )
        })
        .collect();

    let all_active = if active.is_none() {
        "bg-accent text-accent-foreground"
    } else {
        "text-muted-foreground hover:text-foreground hover:bg-accent/50"
    };

    format!(
        r#"
        <nav class="p-4">
            <a href="/library/{lib}" class="block px-4 py-2 text-sm rounded-md mb-2 {all_cls} transition-colors">
                All Components
            </a>
            <div class="text-xs font-semibold text-muted-foreground uppercase tracking-wider px-4 py-2 mt-4">
                Components
            </div>
            {items}
        </nav>
        "#,
        lib = library_id,
        all_cls = all_active,
        items = nav_items
    )
}

fn generate_all_components_preview(library_id: &str, components: &[(&str, &str)]) -> String {
    let cards: String = components
        .iter()
        .map(|(id, name)| {
            let preview = get_component_preview(id);
            format!(
                r#"
                <a href="/library/{lib}/component/{comp}" class="block group">
                    <div class="rounded-lg border border-border overflow-hidden hover:border-primary transition-colors">
                        <div class="p-6 bg-card/50 component-preview min-h-[120px] flex items-center justify-center">
                            {preview}
                        </div>
                        <div class="p-4 border-t border-border bg-card">
                            <h3 class="font-medium group-hover:text-primary transition-colors">{name}</h3>
                        </div>
                    </div>
                </a>
                "#,
                lib = library_id,
                comp = id,
                preview = preview,
                name = name
            )
        })
        .collect();

    format!(
        r#"
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {cards}
        </div>
        "#,
        cards = cards
    )
}

fn generate_component_detail(component: &str) -> String {
    let preview = get_component_preview(component);
    let examples = get_component_examples(component);
    let usage = get_component_usage(component);

    format!(
        r#"
        <div class="space-y-8">
            <!-- Preview -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Preview</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="p-8 bg-card/50 component-preview flex items-center justify-center min-h-[200px]">
                        {preview}
                    </div>
                </div>
            </section>

            <!-- Examples -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Examples</h3>
                <div class="space-y-6">
                    {examples}
                </div>
            </section>

            <!-- Usage -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Usage</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="p-4 bg-muted/30">
                        <pre class="text-sm overflow-x-auto"><code>{usage}</code></pre>
                    </div>
                </div>
            </section>
        </div>
        "#,
        preview = preview,
        examples = examples,
        usage = usage
    )
}

fn get_component_preview(component: &str) -> &'static str {
    match component {
        "button" => r#"
            <div class="flex gap-2">
                <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Button</button>
            </div>
        "#,

        "input" => r#"
            <input type="text" placeholder="Enter text..." class="flex h-9 w-full max-w-xs rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
        "#,

        "card" => r#"
            <div class="rounded-lg border border-border bg-card text-card-foreground shadow-sm w-full max-w-sm">
                <div class="p-4">
                    <h4 class="font-semibold">Card Title</h4>
                    <p class="text-sm text-muted-foreground">Card description</p>
                </div>
            </div>
        "#,

        "checkbox" => r#"
            <div class="flex items-center space-x-2">
                <input type="checkbox" id="preview-cb" class="h-4 w-4 rounded border border-primary">
                <label for="preview-cb" class="text-sm">Accept terms</label>
            </div>
        "#,

        "switch" => r#"
            <div x-data="{ on: false }" class="flex items-center gap-2">
                <button @click="on = !on" :class="on ? 'bg-primary' : 'bg-input'" class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors">
                    <span :class="on ? 'translate-x-5' : 'translate-x-1'" class="inline-block h-3 w-3 rounded-full bg-white transition-transform"></span>
                </button>
                <span class="text-sm">Enabled</span>
            </div>
        "#,

        "badge" => r#"
            <div class="flex gap-2">
                <span class="inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-semibold bg-primary text-primary-foreground">Default</span>
                <span class="inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-semibold bg-secondary text-secondary-foreground">Secondary</span>
                <span class="inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-semibold bg-destructive text-destructive-foreground">Destructive</span>
            </div>
        "#,

        "alert" => r#"
            <div class="relative w-full max-w-md rounded-lg border border-border p-4">
                <h5 class="mb-1 font-medium leading-none">Heads up!</h5>
                <div class="text-sm text-muted-foreground">You can add components to your app using the cli.</div>
            </div>
        "#,

        "avatar" => r#"
            <div class="flex gap-2">
                <div class="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full bg-muted">
                    <span class="flex h-full w-full items-center justify-center text-sm">JD</span>
                </div>
                <div class="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full">
                    <img src="https://github.com/shadcn.png" alt="Avatar" class="aspect-square h-full w-full">
                </div>
            </div>
        "#,

        "progress" => r#"
            <div class="w-full max-w-xs">
                <div class="relative h-2 w-full overflow-hidden rounded-full bg-primary/20">
                    <div class="h-full w-[60%] bg-primary transition-all"></div>
                </div>
            </div>
        "#,

        "skeleton" => r#"
            <div class="flex items-center space-x-4">
                <div class="h-12 w-12 rounded-full bg-muted animate-pulse"></div>
                <div class="space-y-2">
                    <div class="h-4 w-[200px] bg-muted animate-pulse rounded"></div>
                    <div class="h-4 w-[150px] bg-muted animate-pulse rounded"></div>
                </div>
            </div>
        "#,

        "separator" => r#"
            <div class="w-full max-w-xs">
                <div class="space-y-1">
                    <h4 class="text-sm font-medium">Radix Primitives</h4>
                    <p class="text-sm text-muted-foreground">An open-source UI component library.</p>
                </div>
                <div class="my-4 h-[1px] w-full bg-border"></div>
                <div class="flex h-5 items-center space-x-4 text-sm">
                    <div>Blog</div>
                    <div class="h-full w-[1px] bg-border"></div>
                    <div>Docs</div>
                    <div class="h-full w-[1px] bg-border"></div>
                    <div>Source</div>
                </div>
            </div>
        "#,

        "label" => r#"
            <div class="grid w-full max-w-sm items-center gap-1.5">
                <label class="text-sm font-medium leading-none">Email</label>
                <input type="email" placeholder="Email" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
            </div>
        "#,

        "textarea" => r#"
            <textarea placeholder="Type your message here." class="flex min-h-[60px] w-full max-w-sm rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"></textarea>
        "#,

        "select" => r#"
            <div x-data="{ open: false, selected: 'Select a fruit', pos: { top: 0, left: 0, width: 0 } }" class="relative w-[180px]">
                <button x-ref="trigger" @click="let r = $refs.trigger.getBoundingClientRect(); pos = { top: r.bottom + window.scrollY, left: r.left + window.scrollX, width: r.width }; open = !open" class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring">
                    <span x-text="selected"></span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-50"><path d="m6 9 6 6 6-6"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-50 rounded-md border border-border bg-popover text-popover-foreground p-1 shadow-md" :style="'top: ' + pos.top + 'px; left: ' + pos.left + 'px; width: ' + pos.width + 'px;'">
                        <div @click="selected = 'Apple'; open = false" class="relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground">Apple</div>
                        <div @click="selected = 'Banana'; open = false" class="relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground">Banana</div>
                        <div @click="selected = 'Orange'; open = false" class="relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground">Orange</div>
                    </div>
                </template>
            </div>
        "#,

        "tabs" => r#"
            <div x-data="{ tab: 'account' }" class="w-full max-w-md">
                <div class="inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground">
                    <button @click="tab = 'account'" :class="tab === 'account' ? 'bg-background text-foreground shadow-sm' : ''" class="inline-flex items-center justify-center rounded-md px-3 py-1 text-sm font-medium transition-all">Account</button>
                    <button @click="tab = 'password'" :class="tab === 'password' ? 'bg-background text-foreground shadow-sm' : ''" class="inline-flex items-center justify-center rounded-md px-3 py-1 text-sm font-medium transition-all">Password</button>
                </div>
                <div class="mt-4">
                    <div x-show="tab === 'account'" class="text-sm">Account settings content</div>
                    <div x-show="tab === 'password'" x-cloak class="text-sm">Password settings content</div>
                </div>
            </div>
        "#,

        "accordion" => r#"
            <div x-data="{ open: null }" class="w-full max-w-md space-y-1">
                <div class="border-b">
                    <button @click="open = open === 1 ? null : 1" class="flex w-full items-center justify-between py-4 text-sm font-medium transition-all hover:underline">
                        Is it accessible?
                        <svg :class="open === 1 ? 'rotate-180' : ''" class="h-4 w-4 transition-transform" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                    </button>
                    <div x-show="open === 1" x-collapse class="text-sm text-muted-foreground pb-4">Yes. It adheres to the WAI-ARIA design pattern.</div>
                </div>
                <div class="border-b">
                    <button @click="open = open === 2 ? null : 2" class="flex w-full items-center justify-between py-4 text-sm font-medium transition-all hover:underline">
                        Is it styled?
                        <svg :class="open === 2 ? 'rotate-180' : ''" class="h-4 w-4 transition-transform" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                    </button>
                    <div x-show="open === 2" x-collapse class="text-sm text-muted-foreground pb-4">Yes. It comes with default styles that match the other components.</div>
                </div>
            </div>
        "#,

        "dialog" => r#"
            <div x-data="{ open: false }">
                <button @click="open = true" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Open Dialog</button>
                <template x-teleport="body">
                    <div x-show="open" x-cloak class="fixed inset-0 z-50 flex items-center justify-center">
                        <div @click="open = false" class="fixed inset-0 bg-black/80"></div>
                        <div class="relative z-50 w-full max-w-lg rounded-lg border bg-background p-6 shadow-lg">
                            <h2 class="text-lg font-semibold">Edit profile</h2>
                            <p class="text-sm text-muted-foreground mt-2">Make changes to your profile here.</p>
                            <div class="mt-4 flex justify-end gap-2">
                                <button @click="open = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Cancel</button>
                                <button @click="open = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Save</button>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "dropdown" => r#"
            <div x-data="{ open: false, pos: { top: 0, left: 0 } }" class="relative inline-block">
                <button x-ref="trigger" @click="let r = $refs.trigger.getBoundingClientRect(); pos = { top: r.bottom + window.scrollY, left: r.left + window.scrollX }; open = !open" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">
                    Open Menu
                    <svg class="ml-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed w-56 rounded-md border border-border bg-popover text-popover-foreground p-1 shadow-md z-50" :style="'top: ' + pos.top + 'px; left: ' + pos.left + 'px;'">
                        <div class="px-2 py-1.5 text-sm font-semibold">My Account</div>
                        <div class="h-px bg-border my-1"></div>
                        <button class="w-full text-left px-2 py-1.5 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground">Profile</button>
                        <button class="w-full text-left px-2 py-1.5 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground">Settings</button>
                        <div class="h-px bg-border my-1"></div>
                        <button class="w-full text-left px-2 py-1.5 text-sm rounded-sm hover:bg-accent text-destructive">Log out</button>
                    </div>
                </template>
            </div>
        "#,

        "popover" => r#"
            <div x-data="{ open: false, pos: { top: 0, left: 0 } }" class="relative inline-block">
                <button x-ref="trigger" @click="let r = $refs.trigger.getBoundingClientRect(); pos = { top: r.bottom + window.scrollY, left: r.left + window.scrollX }; open = !open" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Open Popover</button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed w-80 rounded-md border border-border bg-popover text-popover-foreground p-4 shadow-md z-50" :style="'top: ' + pos.top + 'px; left: ' + pos.left + 'px;'">
                        <div class="grid gap-4">
                            <div class="space-y-2">
                                <h4 class="font-medium leading-none">Dimensions</h4>
                                <p class="text-sm text-muted-foreground">Set the dimensions for the layer.</p>
                            </div>
                            <div class="grid gap-2">
                                <div class="grid grid-cols-3 items-center gap-4">
                                    <label class="text-sm">Width</label>
                                    <input class="col-span-2 h-8 rounded-md border border-input px-2 text-sm bg-transparent" value="100%">
                                </div>
                                <div class="grid grid-cols-3 items-center gap-4">
                                    <label class="text-sm">Height</label>
                                    <input class="col-span-2 h-8 rounded-md border border-input px-2 text-sm bg-transparent" value="25px">
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "tooltip" => r#"
            <div x-data="{ show: false }" class="relative inline-block">
                <button @mouseenter="show = true" @mouseleave="show = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Hover me</button>
                <div x-show="show" x-cloak class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-md shadow-md whitespace-nowrap">
                    Add to library
                </div>
            </div>
        "#,

        "slider" => r#"
            <div x-data="{ value: 50 }" class="w-full max-w-xs">
                <input type="range" x-model="value" class="w-full h-2 bg-primary/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:bg-primary [&::-webkit-slider-thumb]:rounded-full">
                <div class="text-sm text-muted-foreground mt-2">Value: <span x-text="value"></span></div>
            </div>
        "#,

        "radio-group" => r#"
            <div x-data="{ selected: 'default' }" class="space-y-2">
                <div class="flex items-center space-x-2">
                    <input type="radio" id="r1" name="size" value="default" x-model="selected" class="h-4 w-4">
                    <label for="r1" class="text-sm">Default</label>
                </div>
                <div class="flex items-center space-x-2">
                    <input type="radio" id="r2" name="size" value="comfortable" x-model="selected" class="h-4 w-4">
                    <label for="r2" class="text-sm">Comfortable</label>
                </div>
                <div class="flex items-center space-x-2">
                    <input type="radio" id="r3" name="size" value="compact" x-model="selected" class="h-4 w-4">
                    <label for="r3" class="text-sm">Compact</label>
                </div>
            </div>
        "#,

        "toggle" => r#"
            <div x-data="{ pressed: false }">
                <button @click="pressed = !pressed" :class="pressed ? 'bg-accent' : ''" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-3 border border-input hover:bg-accent hover:text-accent-foreground transition-colors">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4v6a6 6 0 0 0 12 0V4"/><line x1="4" x2="20" y1="20" y2="20"/></svg>
                </button>
            </div>
        "#,

        "table" => r#"
            <div class="w-full max-w-lg rounded-md border">
                <table class="w-full caption-bottom text-sm">
                    <thead class="[&_tr]:border-b">
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <th class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">Invoice</th>
                            <th class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">Status</th>
                            <th class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">Amount</th>
                        </tr>
                    </thead>
                    <tbody class="[&_tr:last-child]:border-0">
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <td class="p-4 align-middle">INV001</td>
                            <td class="p-4 align-middle">Paid</td>
                            <td class="p-4 align-middle text-right">$250.00</td>
                        </tr>
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <td class="p-4 align-middle">INV002</td>
                            <td class="p-4 align-middle">Pending</td>
                            <td class="p-4 align-middle text-right">$150.00</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        "#,

        "breadcrumb" => r##"
            <nav class="flex">
                <ol class="flex items-center gap-2 text-sm">
                    <li><a href="#" class="text-muted-foreground hover:text-foreground">Home</a></li>
                    <li class="text-muted-foreground">/</li>
                    <li><a href="#" class="text-muted-foreground hover:text-foreground">Components</a></li>
                    <li class="text-muted-foreground">/</li>
                    <li class="text-foreground">Breadcrumb</li>
                </ol>
            </nav>
        "##,

        "form" => r#"
            <form class="w-full max-w-sm space-y-4">
                <div class="space-y-2">
                    <label class="text-sm font-medium">Username</label>
                    <input type="text" placeholder="Enter username" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                    <p class="text-xs text-muted-foreground">This is your public display name.</p>
                </div>
                <button type="submit" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Submit</button>
            </form>
        "#,

        "collapsible" => r#"
            <div x-data="{ open: false }" class="w-full max-w-md">
                <div class="flex items-center justify-between">
                    <h4 class="text-sm font-semibold">@peduarte starred 3 repositories</h4>
                    <button @click="open = !open" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 w-9 border border-input hover:bg-accent">
                        <svg :class="open ? 'rotate-180' : ''" class="h-4 w-4 transition-transform" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                    </button>
                </div>
                <div class="rounded-md border px-4 py-2 font-mono text-sm mt-2">@radix-ui/primitives</div>
                <div x-show="open" x-collapse class="mt-2 space-y-2">
                    <div class="rounded-md border px-4 py-2 font-mono text-sm">@radix-ui/colors</div>
                    <div class="rounded-md border px-4 py-2 font-mono text-sm">@stitches/react</div>
                </div>
            </div>
        "#,

        "combobox" => r#"
            <div x-data="{ open: false, search: '', selected: '', pos: { top: 0, left: 0, width: 0 } }" class="relative w-[200px]">
                <button x-ref="trigger" @click="let r = $refs.trigger.getBoundingClientRect(); pos = { top: r.bottom + window.scrollY, left: r.left + window.scrollX, width: r.width }; open = !open" class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm">
                    <span x-text="selected || 'Select framework...'"></span>
                    <svg class="h-4 w-4 opacity-50" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-50 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="'top: ' + pos.top + 'px; left: ' + pos.left + 'px; width: ' + pos.width + 'px;'">
                        <input x-model="search" placeholder="Search..." class="w-full border-b border-border px-3 py-2 text-sm outline-none bg-transparent placeholder:text-muted-foreground">
                        <div class="p-1">
                            <div @click="selected = 'Next.js'; open = false" x-show="'next.js'.includes(search.toLowerCase())" class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground cursor-pointer">Next.js</div>
                            <div @click="selected = 'SvelteKit'; open = false" x-show="'sveltekit'.includes(search.toLowerCase())" class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground cursor-pointer">SvelteKit</div>
                            <div @click="selected = 'Nuxt'; open = false" x-show="'nuxt'.includes(search.toLowerCase())" class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent hover:text-accent-foreground cursor-pointer">Nuxt</div>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "alert-dialog" => r#"
            <div x-data="{ open: false }">
                <button @click="open = true" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90">Delete Account</button>
                <template x-teleport="body">
                    <div x-show="open" x-cloak class="fixed inset-0 z-50 flex items-center justify-center">
                        <div class="fixed inset-0 bg-black/80"></div>
                        <div class="relative z-50 w-full max-w-lg rounded-lg border bg-background p-6 shadow-lg">
                            <h2 class="text-lg font-semibold">Are you absolutely sure?</h2>
                            <p class="text-sm text-muted-foreground mt-2">This action cannot be undone. This will permanently delete your account.</p>
                            <div class="mt-4 flex justify-end gap-2">
                                <button @click="open = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Cancel</button>
                                <button @click="open = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90">Delete</button>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        _ => r#"<div class="text-muted-foreground text-sm">Component preview</div>"#,
    }
}

fn get_component_examples(component: &str) -> String {
    match component {
        "button" => r#"
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">Variants</h4>
                <div class="flex flex-wrap gap-2">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Primary</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-secondary text-secondary-foreground hover:bg-secondary/80">Secondary</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90">Destructive</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground">Outline</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 hover:bg-accent hover:text-accent-foreground">Ghost</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 underline-offset-4 hover:underline">Link</button>
                </div>
            </div>
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">Sizes</h4>
                <div class="flex flex-wrap items-center gap-2">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-8 px-3 text-xs bg-primary text-primary-foreground hover:bg-primary/90">Small</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Default</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-8 bg-primary text-primary-foreground hover:bg-primary/90">Large</button>
                </div>
            </div>
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">Disabled</h4>
                <button disabled class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground opacity-50 cursor-not-allowed">Disabled</button>
            </div>
        "#.to_string(),

        "input" => r#"
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">With Label</h4>
                <div class="grid w-full max-w-sm items-center gap-1.5">
                    <label class="text-sm font-medium">Email</label>
                    <input type="email" placeholder="Email" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                </div>
            </div>
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">Disabled</h4>
                <input disabled type="text" placeholder="Email" class="flex h-9 w-full max-w-sm rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50">
            </div>
            <div class="rounded-lg border border-border p-6 space-y-4">
                <h4 class="font-medium mb-4">With Error</h4>
                <div class="grid w-full max-w-sm items-center gap-1.5">
                    <label class="text-sm font-medium">Email</label>
                    <input type="email" placeholder="Email" class="flex h-9 w-full rounded-md border border-destructive bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-destructive">
                    <p class="text-sm text-destructive">Please enter a valid email address.</p>
                </div>
            </div>
        "#.to_string(),

        _ => format!(
            r#"<div class="rounded-lg border border-border p-6">
                <p class="text-muted-foreground">Additional examples for this component</p>
            </div>"#,
        ),
    }
}

fn get_component_usage(component: &str) -> String {
    match component {
        "button" => r#"@import components/button.wtz as button

// Primary button
&lt;@button::primary&gt;Click me&lt;/@&gt;

// With variant and size
&lt;@button
    variant=button::Variant::Secondary
    size=button::Size::Lg
    disabled=false
&gt;
    Large Secondary
&lt;/@&gt;"#.to_string(),

        "input" => r#"@import components/input.wtz as input

// Basic input
&lt;@input
    name="email"
    placeholder=Some("Enter email")
    type_attr="email"
/&gt;

// With label
&lt;@input::labeled
    label="Email"
    name="email"
    required=true
    error=None
/&gt;"#.to_string(),

        "card" => r#"@import components/card.wtz as card

// Simple card
&lt;@card&gt;
    &lt;@card::header&gt;
        &lt;@card::title&gt;Card Title&lt;/@&gt;
        &lt;@card::description&gt;Card description&lt;/@&gt;
    &lt;/@&gt;
    &lt;@card::content&gt;
        Content here
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        _ => {
            let comp_name = component.replace('-', "_");
            format!(
                r#"@import components/{}.wtz as {}

// Basic usage
&lt;@{} ... /&gt;

// See component source for options"#,
                component, comp_name, comp_name
            )
        }
    }
}
