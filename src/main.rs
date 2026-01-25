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
            background-color: hsl(var(--background));
            background-image: radial-gradient(hsl(var(--border)) 1px, transparent 1px);
            background-size: 16px 16px;
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
            ("ajax-select", "Ajax Select"),
            ("alert", "Alert"),
            ("alert-dialog", "Alert Dialog"),
            ("aspect-ratio", "Aspect Ratio"),
            ("avatar", "Avatar"),
            ("badge", "Badge"),
            ("breadcrumb", "Breadcrumb"),
            ("button", "Button"),
            ("calendar", "Calendar"),
            ("card", "Card"),
            ("carousel", "Carousel"),
            ("checkbox", "Checkbox"),
            ("collapsible", "Collapsible"),
            ("combobox", "Combobox"),
            ("command", "Command"),
            ("context-menu", "Context Menu"),
            ("date-picker", "Date Picker"),
            ("datetime-picker", "DateTime Picker"),
            ("dialog", "Dialog"),
            ("drawer", "Drawer"),
            ("dropdown", "Dropdown"),
            ("duration-input", "Duration Input"),
            ("file-upload", "File Upload"),
            ("flash-messages", "Flash Messages"),
            ("form", "Form"),
            ("formatted-number", "Formatted Number"),
            ("hover-card", "Hover Card"),
            ("input", "Input"),
            ("input-otp", "Input OTP"),
            ("label", "Label"),
            ("menubar", "Menubar"),
            ("minmax-editor", "Min/Max Editor"),
            ("navigation-menu", "Navigation Menu"),
            ("numeric-input", "Numeric Input"),
            ("pagination", "Pagination"),
            ("password-input", "Password Input"),
            ("popover", "Popover"),
            ("progress", "Progress"),
            ("radio-group", "Radio Group"),
            ("resizable", "Resizable"),
            ("scroll-area", "Scroll Area"),
            ("searchable-select", "Searchable Select"),
            ("select", "Select"),
            ("separator", "Separator"),
            ("sheet", "Sheet"),
            ("skeleton", "Skeleton"),
            ("slider", "Slider"),
            ("switch", "Switch"),
            ("table", "Table"),
            ("tabs", "Tabs"),
            ("textarea", "Textarea"),
            ("time-picker", "Time Picker"),
            ("toast", "Toast"),
            ("toggle", "Toggle"),
            ("toggle-group", "Toggle Group"),
            ("tooltip", "Tooltip"),
            ("validation-errors", "Validation Errors"),
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
            <div class="flex flex-wrap gap-4">
                <button class="inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90 transition-colors">
                    <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                    Primary
                </button>
                <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80 transition-colors">Secondary</button>
                <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors">Outline</button>
                <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 hover:bg-accent hover:text-accent-foreground transition-colors">Ghost</button>
                <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90 transition-colors">Destructive</button>
            </div>
        "#,

        "input" => r#"
            <input type="text" placeholder="Enter text..." class="flex h-9 w-full max-w-xs rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
        "#,

        "card" => r#"
            <div class="rounded-xl border border-border bg-card text-card-foreground shadow w-full max-w-sm">
                <div class="flex flex-col space-y-1.5 p-6">
                    <h3 class="font-semibold leading-none tracking-tight">Create project</h3>
                    <p class="text-sm text-muted-foreground">Deploy your new project in one-click.</p>
                </div>
                <div class="p-6 pt-0 space-y-4">
                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none">Name</label>
                        <input type="text" placeholder="Name of your project" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                    </div>
                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none">Framework</label>
                        <div class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm">
                            <span>Select</span>
                            <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                        </div>
                    </div>
                </div>
                <div class="flex items-center p-6 pt-0 justify-between">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground transition-colors">Cancel</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90 transition-colors">Deploy</button>
                </div>
            </div>
        "#,

        "checkbox" => r#"
            <div class="space-y-4">
                <div x-data="{ checked: true }" class="flex items-center space-x-3">
                    <button type="button" @click="checked = !checked" :class="checked ? 'bg-primary border-primary' : 'border-input'" class="h-4 w-4 shrink-0 rounded-sm border shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors flex items-center justify-center">
                        <svg x-show="checked" class="h-3 w-3 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                    </button>
                    <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Accept terms and conditions</label>
                </div>
                <div x-data="{ checked: false }" class="flex items-center space-x-3">
                    <button type="button" @click="checked = !checked" :class="checked ? 'bg-primary border-primary' : 'border-input'" class="h-4 w-4 shrink-0 rounded-sm border shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors flex items-center justify-center">
                        <svg x-show="checked" class="h-3 w-3 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                    </button>
                    <label class="text-sm font-medium leading-none">Send me marketing emails</label>
                </div>
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
            <div class="flex flex-wrap gap-2">
                <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-primary text-primary-foreground">Default</span>
                <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-secondary text-secondary-foreground">Secondary</span>
                <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-destructive text-destructive-foreground">Destructive</span>
                <span class="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-semibold">Outline</span>
            </div>
        "#,

        "alert" => r#"
            <div class="space-y-4 w-full max-w-md">
                <div class="relative rounded-lg border border-border bg-background p-4 [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg+div]:translate-y-[-3px] [&:has(svg)]:pl-11">
                    <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 16h.01"/><path d="M12 8v4"/><circle cx="12" cy="12" r="10"/></svg>
                    <h5 class="mb-1 font-medium leading-none tracking-tight">Heads up!</h5>
                    <div class="text-sm text-muted-foreground">You can add components to your app using the CLI.</div>
                </div>
                <div class="relative rounded-lg border border-destructive/50 bg-destructive/10 text-destructive p-4 [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg+div]:translate-y-[-3px] [&:has(svg)]:pl-11">
                    <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                    <h5 class="mb-1 font-medium leading-none tracking-tight">Error</h5>
                    <div class="text-sm opacity-90">Your session has expired. Please log in again.</div>
                </div>
            </div>
        "#,

        "avatar" => r#"
            <div class="flex items-center gap-4">
                <div class="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full">
                    <img src="https://github.com/shadcn.png" alt="@shadcn" class="aspect-square h-full w-full object-cover">
                </div>
                <div class="relative flex h-12 w-12 shrink-0 overflow-hidden rounded-full bg-gradient-to-br from-violet-500 to-fuchsia-500">
                    <span class="flex h-full w-full items-center justify-center text-sm font-medium text-white">JD</span>
                </div>
                <div class="relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full bg-muted">
                    <span class="flex h-full w-full items-center justify-center text-sm font-medium">CN</span>
                </div>
                <div class="relative flex h-8 w-8 shrink-0 overflow-hidden rounded-full ring-2 ring-primary ring-offset-2 ring-offset-background">
                    <img src="https://github.com/vercel.png" alt="@vercel" class="aspect-square h-full w-full object-cover">
                </div>
            </div>
        "#,

        "progress" => r#"
            <div x-data="{ progress: 60 }" x-init="setInterval(() => { progress = progress >= 100 ? 0 : progress + 10 }, 800)" class="w-full max-w-xs space-y-4">
                <div class="space-y-2">
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Progress</span>
                        <span x-text="progress + '%'" class="font-medium"></span>
                    </div>
                    <div class="relative h-2 w-full overflow-hidden rounded-full bg-primary/20">
                        <div class="h-full bg-primary transition-all duration-500 ease-out" :style="`width: ${progress}%`"></div>
                    </div>
                </div>
                <div class="space-y-2">
                    <div class="text-sm text-muted-foreground">Indeterminate</div>
                    <div class="relative h-2 w-full overflow-hidden rounded-full bg-primary/20">
                        <div class="h-full w-1/3 bg-primary animate-[progress_1s_ease-in-out_infinite] absolute"></div>
                    </div>
                </div>
            </div>
            <style>
                @keyframes progress {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(400%); }
                }
            </style>
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
            <div x-data="{
                show: false,
                pos: { x: 0, y: 0 },
                updatePos() {
                    const rect = this.$refs.trigger.getBoundingClientRect();
                    this.pos.x = rect.left + rect.width / 2;
                    this.pos.y = rect.top - 8;
                }
            }" class="inline-block">
                <button x-ref="trigger" @mouseenter="updatePos(); show = true" @mouseleave="show = false" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Hover me</button>
                <template x-teleport="body">
                    <div x-show="show" x-cloak
                         :style="`left: ${pos.x}px; top: ${pos.y}px; transform: translate(-50%, -100%);`"
                         class="fixed px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-md shadow-md whitespace-nowrap z-[9999]">
                        Add to library
                    </div>
                </template>
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
            <div class="w-full max-w-lg rounded-lg border bg-card">
                <table class="w-full caption-bottom text-sm">
                    <thead class="[&_tr]:border-b">
                        <tr class="border-b transition-colors">
                            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Invoice</th>
                            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Status</th>
                            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Method</th>
                            <th class="h-12 px-4 text-right align-middle font-medium text-muted-foreground">Amount</th>
                        </tr>
                    </thead>
                    <tbody class="[&_tr:last-child]:border-0">
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <td class="p-4 align-middle font-medium">INV001</td>
                            <td class="p-4 align-middle"><span class="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300">Paid</span></td>
                            <td class="p-4 align-middle text-muted-foreground">Credit Card</td>
                            <td class="p-4 align-middle text-right font-medium">$250.00</td>
                        </tr>
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <td class="p-4 align-middle font-medium">INV002</td>
                            <td class="p-4 align-middle"><span class="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300">Pending</span></td>
                            <td class="p-4 align-middle text-muted-foreground">PayPal</td>
                            <td class="p-4 align-middle text-right font-medium">$150.00</td>
                        </tr>
                        <tr class="border-b transition-colors hover:bg-muted/50">
                            <td class="p-4 align-middle font-medium">INV003</td>
                            <td class="p-4 align-middle"><span class="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300">Unpaid</span></td>
                            <td class="p-4 align-middle text-muted-foreground">Bank Transfer</td>
                            <td class="p-4 align-middle text-right font-medium">$350.00</td>
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

        "ajax-select" => r#"
            <div x-data="{ open: false, search: '', loading: false, selected: '' }" class="relative w-[200px]">
                <button @click="open = !open" class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm">
                    <span x-text="selected || 'Search users...'"></span>
                    <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                </button>
            </div>
        "#,

        "aspect-ratio" => r#"
            <div class="w-full max-w-[200px]">
                <div class="relative" style="aspect-ratio: 16/9;">
                    <img src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo" class="h-full w-full rounded-md object-cover">
                </div>
            </div>
        "#,

        "calendar" => r#"
            <div class="p-3 rounded-md border bg-card text-card-foreground">
                <div class="flex items-center justify-between mb-4">
                    <button class="h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
                    </button>
                    <div class="text-sm font-medium">January 2026</div>
                    <button class="h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
                    </button>
                </div>
                <div class="grid grid-cols-7 gap-1 text-center text-xs">
                    <div class="text-muted-foreground">Su</div><div class="text-muted-foreground">Mo</div><div class="text-muted-foreground">Tu</div><div class="text-muted-foreground">We</div><div class="text-muted-foreground">Th</div><div class="text-muted-foreground">Fr</div><div class="text-muted-foreground">Sa</div>
                    <div class="h-8 w-8"></div><div class="h-8 w-8"></div><div class="h-8 w-8"></div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">1</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">2</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">3</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">4</div>
                    <div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">5</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">6</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">7</div><div class="h-8 w-8 flex items-center justify-center rounded-md bg-primary text-primary-foreground">8</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">9</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">10</div><div class="h-8 w-8 flex items-center justify-center rounded-md hover:bg-accent">11</div>
                </div>
            </div>
        "#,

        "carousel" => r#"
            <div x-data="{ current: 0, total: 5 }" class="relative w-full max-w-sm">
                <div class="overflow-hidden rounded-lg">
                    <div class="flex transition-transform duration-500 ease-out" :style="`transform: translateX(-${current * 100}%)`">
                        <div class="w-full flex-shrink-0 p-1">
                            <div class="rounded-xl border bg-card p-6 aspect-square flex flex-col items-center justify-center">
                                <span class="text-4xl font-semibold">1</span>
                                <span class="text-sm text-muted-foreground mt-2">Slide content</span>
                            </div>
                        </div>
                        <div class="w-full flex-shrink-0 p-1">
                            <div class="rounded-xl border bg-card p-6 aspect-square flex flex-col items-center justify-center">
                                <span class="text-4xl font-semibold">2</span>
                                <span class="text-sm text-muted-foreground mt-2">Slide content</span>
                            </div>
                        </div>
                        <div class="w-full flex-shrink-0 p-1">
                            <div class="rounded-xl border bg-card p-6 aspect-square flex flex-col items-center justify-center">
                                <span class="text-4xl font-semibold">3</span>
                                <span class="text-sm text-muted-foreground mt-2">Slide content</span>
                            </div>
                        </div>
                        <div class="w-full flex-shrink-0 p-1">
                            <div class="rounded-xl border bg-card p-6 aspect-square flex flex-col items-center justify-center">
                                <span class="text-4xl font-semibold">4</span>
                                <span class="text-sm text-muted-foreground mt-2">Slide content</span>
                            </div>
                        </div>
                        <div class="w-full flex-shrink-0 p-1">
                            <div class="rounded-xl border bg-card p-6 aspect-square flex flex-col items-center justify-center">
                                <span class="text-4xl font-semibold">5</span>
                                <span class="text-sm text-muted-foreground mt-2">Slide content</span>
                            </div>
                        </div>
                    </div>
                </div>
                <button @click="current = (current - 1 + total) % total" class="absolute left-2 top-1/2 -translate-y-1/2 h-8 w-8 rounded-full bg-background/90 border shadow-sm flex items-center justify-center hover:bg-accent transition-colors">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
                </button>
                <button @click="current = (current + 1) % total" class="absolute right-2 top-1/2 -translate-y-1/2 h-8 w-8 rounded-full bg-background/90 border shadow-sm flex items-center justify-center hover:bg-accent transition-colors">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
                </button>
                <div class="flex justify-center gap-1.5 mt-4">
                    <template x-for="i in total" :key="i">
                        <button @click="current = i - 1" :class="current === i - 1 ? 'bg-primary' : 'bg-primary/30'" class="h-2 w-2 rounded-full transition-colors"></button>
                    </template>
                </div>
            </div>
        "#,

        "command" => r#"
            <div class="w-full max-w-sm rounded-lg border shadow-md">
                <div class="flex items-center border-b px-3">
                    <svg class="mr-2 h-4 w-4 shrink-0 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                    <input class="flex h-10 w-full bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground" placeholder="Type a command or search...">
                </div>
                <div class="p-1">
                    <div class="px-2 py-1.5 text-xs font-medium text-muted-foreground">Suggestions</div>
                    <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Calendar</div>
                    <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Search Emoji</div>
                    <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Calculator</div>
                </div>
            </div>
        "#,

        "context-menu" => r#"
            <div x-data="{ open: false, pos: { x: 0, y: 0 } }" class="relative">
                <div @contextmenu.prevent="pos = { x: $event.clientX, y: $event.clientY }; open = true" class="flex h-[150px] w-[300px] items-center justify-center rounded-lg border border-dashed border-border bg-muted/20 text-sm text-muted-foreground select-none cursor-context-menu transition-colors hover:bg-muted/40">
                    Right click here
                </div>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-50 min-w-[160px] rounded-md border bg-popover p-1 text-popover-foreground shadow-md" :style="`left: ${pos.x}px; top: ${pos.y}px;`">
                        <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2">
                            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
                            New File
                        </div>
                        <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2">
                            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"/><polyline points="14 2 14 8 20 8"/><path d="M3 15h6"/><path d="M6 12v6"/></svg>
                            New Folder
                        </div>
                        <div class="h-px bg-border my-1"></div>
                        <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2">
                            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                            Copy
                            <span class="ml-auto text-xs text-muted-foreground">⌘C</span>
                        </div>
                        <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2">
                            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15V6"/><path d="M18.5 18a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z"/><path d="M12 12H3"/><path d="M16 6H3"/><path d="M12 18H3"/></svg>
                            Paste
                            <span class="ml-auto text-xs text-muted-foreground">⌘V</span>
                        </div>
                        <div class="h-px bg-border my-1"></div>
                        <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2 text-destructive">
                            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                            Delete
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "date-picker" => r#"
            <div x-data="{ open: false, selected: '', pos: { top: 0, left: 0 } }" class="relative">
                <button x-ref="trigger" @click="let r = $refs.trigger.getBoundingClientRect(); pos = { top: r.bottom + window.scrollY + 4, left: r.left + window.scrollX }; open = !open" class="flex h-9 w-[200px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                    <span :class="selected ? '' : 'text-muted-foreground'" x-text="selected || 'Pick a date'"></span>
                    <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-50 p-3 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="'top: ' + pos.top + 'px; left: ' + pos.left + 'px;'">
                        <div class="flex items-center justify-between mb-3">
                            <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                            <div class="text-sm font-medium">January 2026</div>
                            <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                        </div>
                        <div class="grid grid-cols-7 gap-1 text-center text-xs mb-1">
                            <div class="text-muted-foreground p-1">Su</div><div class="text-muted-foreground p-1">Mo</div><div class="text-muted-foreground p-1">Tu</div><div class="text-muted-foreground p-1">We</div><div class="text-muted-foreground p-1">Th</div><div class="text-muted-foreground p-1">Fr</div><div class="text-muted-foreground p-1">Sa</div>
                        </div>
                        <div class="grid grid-cols-7 gap-1 text-center text-sm">
                            <div class="p-2"></div><div class="p-2"></div><div class="p-2"></div><button type="button" @click="selected = 'Jan 1, 2026'; open = false" class="p-2 rounded hover:bg-accent">1</button><button type="button" @click="selected = 'Jan 2, 2026'; open = false" class="p-2 rounded hover:bg-accent">2</button><button type="button" @click="selected = 'Jan 3, 2026'; open = false" class="p-2 rounded hover:bg-accent">3</button><button type="button" @click="selected = 'Jan 4, 2026'; open = false" class="p-2 rounded hover:bg-accent">4</button>
                            <button type="button" @click="selected = 'Jan 5, 2026'; open = false" class="p-2 rounded hover:bg-accent">5</button><button type="button" @click="selected = 'Jan 6, 2026'; open = false" class="p-2 rounded hover:bg-accent">6</button><button type="button" @click="selected = 'Jan 7, 2026'; open = false" class="p-2 rounded hover:bg-accent">7</button><button type="button" @click="selected = 'Jan 8, 2026'; open = false" class="p-2 rounded bg-primary text-primary-foreground">8</button><button type="button" @click="selected = 'Jan 9, 2026'; open = false" class="p-2 rounded hover:bg-accent">9</button><button type="button" @click="selected = 'Jan 10, 2026'; open = false" class="p-2 rounded hover:bg-accent">10</button><button type="button" @click="selected = 'Jan 11, 2026'; open = false" class="p-2 rounded hover:bg-accent">11</button>
                            <button type="button" @click="selected = 'Jan 12, 2026'; open = false" class="p-2 rounded hover:bg-accent">12</button><button type="button" @click="selected = 'Jan 13, 2026'; open = false" class="p-2 rounded hover:bg-accent">13</button><button type="button" @click="selected = 'Jan 14, 2026'; open = false" class="p-2 rounded hover:bg-accent">14</button><button type="button" @click="selected = 'Jan 15, 2026'; open = false" class="p-2 rounded hover:bg-accent">15</button><button type="button" @click="selected = 'Jan 16, 2026'; open = false" class="p-2 rounded hover:bg-accent">16</button><button type="button" @click="selected = 'Jan 17, 2026'; open = false" class="p-2 rounded hover:bg-accent">17</button><button type="button" @click="selected = 'Jan 18, 2026'; open = false" class="p-2 rounded hover:bg-accent">18</button>
                            <button type="button" @click="selected = 'Jan 19, 2026'; open = false" class="p-2 rounded hover:bg-accent">19</button><button type="button" @click="selected = 'Jan 20, 2026'; open = false" class="p-2 rounded hover:bg-accent">20</button><button type="button" @click="selected = 'Jan 21, 2026'; open = false" class="p-2 rounded hover:bg-accent">21</button><button type="button" @click="selected = 'Jan 22, 2026'; open = false" class="p-2 rounded hover:bg-accent">22</button><button type="button" @click="selected = 'Jan 23, 2026'; open = false" class="p-2 rounded hover:bg-accent">23</button><button type="button" @click="selected = 'Jan 24, 2026'; open = false" class="p-2 rounded hover:bg-accent">24</button><button type="button" @click="selected = 'Jan 25, 2026'; open = false" class="p-2 rounded hover:bg-accent">25</button>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "datetime-picker" => r#"
            <div x-data="{ dateOpen: false, timeOpen: false, selectedDate: '', selectedTime: '', datePos: { top: 0, left: 0 }, timePos: { top: 0, left: 0 } }" class="flex gap-2">
                <div class="relative">
                    <button x-ref="dateBtn" @click="let r = $refs.dateBtn.getBoundingClientRect(); datePos = { top: r.bottom + window.scrollY + 4, left: r.left + window.scrollX }; dateOpen = !dateOpen; timeOpen = false" class="flex h-9 w-[140px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                        <span :class="selectedDate ? '' : 'text-muted-foreground'" x-text="selectedDate || 'Date'"></span>
                        <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="dateOpen" @click.away="dateOpen = false" x-cloak class="fixed z-50 p-3 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="'top: ' + datePos.top + 'px; left: ' + datePos.left + 'px;'">
                            <div class="flex items-center justify-between mb-3">
                                <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                                <div class="text-sm font-medium">January 2026</div>
                                <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                            </div>
                            <div class="grid grid-cols-7 gap-1 text-center text-xs mb-1">
                                <div class="text-muted-foreground p-1">Su</div><div class="text-muted-foreground p-1">Mo</div><div class="text-muted-foreground p-1">Tu</div><div class="text-muted-foreground p-1">We</div><div class="text-muted-foreground p-1">Th</div><div class="text-muted-foreground p-1">Fr</div><div class="text-muted-foreground p-1">Sa</div>
                            </div>
                            <div class="grid grid-cols-7 gap-1 text-center text-sm">
                                <div class="p-2"></div><div class="p-2"></div><div class="p-2"></div><button type="button" @click="selectedDate = 'Jan 1'; dateOpen = false" class="p-2 rounded hover:bg-accent">1</button><button type="button" @click="selectedDate = 'Jan 2'; dateOpen = false" class="p-2 rounded hover:bg-accent">2</button><button type="button" @click="selectedDate = 'Jan 3'; dateOpen = false" class="p-2 rounded hover:bg-accent">3</button><button type="button" @click="selectedDate = 'Jan 4'; dateOpen = false" class="p-2 rounded hover:bg-accent">4</button>
                                <button type="button" @click="selectedDate = 'Jan 5'; dateOpen = false" class="p-2 rounded hover:bg-accent">5</button><button type="button" @click="selectedDate = 'Jan 6'; dateOpen = false" class="p-2 rounded hover:bg-accent">6</button><button type="button" @click="selectedDate = 'Jan 7'; dateOpen = false" class="p-2 rounded hover:bg-accent">7</button><button type="button" @click="selectedDate = 'Jan 8'; dateOpen = false" class="p-2 rounded bg-primary text-primary-foreground">8</button><button type="button" @click="selectedDate = 'Jan 9'; dateOpen = false" class="p-2 rounded hover:bg-accent">9</button><button type="button" @click="selectedDate = 'Jan 10'; dateOpen = false" class="p-2 rounded hover:bg-accent">10</button><button type="button" @click="selectedDate = 'Jan 11'; dateOpen = false" class="p-2 rounded hover:bg-accent">11</button>
                            </div>
                        </div>
                    </template>
                </div>
                <div class="relative">
                    <button x-ref="timeBtn" @click="let r = $refs.timeBtn.getBoundingClientRect(); timePos = { top: r.bottom + window.scrollY + 4, left: r.left + window.scrollX }; timeOpen = !timeOpen; dateOpen = false" class="flex h-9 w-[100px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                        <span :class="selectedTime ? '' : 'text-muted-foreground'" x-text="selectedTime || 'Time'"></span>
                        <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="timeOpen" @click.away="timeOpen = false" x-cloak class="fixed z-50 p-2 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="'top: ' + timePos.top + 'px; left: ' + timePos.left + 'px;'">
                            <div class="grid grid-cols-4 gap-1 text-sm">
                                <button type="button" @click="selectedTime = '9:00 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">9:00</button>
                                <button type="button" @click="selectedTime = '9:30 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">9:30</button>
                                <button type="button" @click="selectedTime = '10:00 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">10:00</button>
                                <button type="button" @click="selectedTime = '10:30 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">10:30</button>
                                <button type="button" @click="selectedTime = '11:00 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">11:00</button>
                                <button type="button" @click="selectedTime = '11:30 AM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">11:30</button>
                                <button type="button" @click="selectedTime = '12:00 PM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">12:00</button>
                                <button type="button" @click="selectedTime = '12:30 PM'; timeOpen = false" class="px-3 py-2 rounded hover:bg-accent">12:30</button>
                            </div>
                        </div>
                    </template>
                </div>
            </div>
        "#,

        "drawer" => r#"
            <div x-data="{ open: false }">
                <button @click="open = true" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Open Drawer</button>
                <template x-teleport="body">
                    <div x-show="open" x-cloak class="fixed inset-0 z-50">
                        <div class="fixed inset-0 bg-black/80" @click="open = false"></div>
                        <div x-show="open" class="fixed inset-x-0 bottom-0 z-50 rounded-t-xl border bg-background p-4">
                            <div class="mx-auto mb-4 h-2 w-24 rounded-full bg-muted"></div>
                            <h3 class="text-lg font-semibold text-center">Drawer Title</h3>
                            <p class="text-sm text-muted-foreground text-center mt-2">Drawer content goes here.</p>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "duration-input" => r#"
            <div class="space-y-4">
                <div x-data="{ hours: '02', minutes: '30', seconds: '00' }" class="flex items-center gap-1">
                    <div class="flex flex-col items-center">
                        <span class="text-xs text-muted-foreground mb-1">HH</span>
                        <input type="text" x-model="hours" maxlength="2" @input="hours = $event.target.value.replace(/\D/g, '').slice(0,2).padStart(2, '0')" class="w-12 h-10 text-center rounded-md border border-input bg-transparent text-lg font-mono">
                    </div>
                    <span class="text-xl font-bold mt-4">:</span>
                    <div class="flex flex-col items-center">
                        <span class="text-xs text-muted-foreground mb-1">MM</span>
                        <input type="text" x-model="minutes" maxlength="2" @input="minutes = Math.min(59, $event.target.value.replace(/\D/g, '')).toString().padStart(2, '0')" class="w-12 h-10 text-center rounded-md border border-input bg-transparent text-lg font-mono">
                    </div>
                    <span class="text-xl font-bold mt-4">:</span>
                    <div class="flex flex-col items-center">
                        <span class="text-xs text-muted-foreground mb-1">SS</span>
                        <input type="text" x-model="seconds" maxlength="2" @input="seconds = Math.min(59, $event.target.value.replace(/\D/g, '')).toString().padStart(2, '0')" class="w-12 h-10 text-center rounded-md border border-input bg-transparent text-lg font-mono">
                    </div>
                </div>
                <div x-data="{ days: '01', hours: '12' }" class="flex items-center gap-1">
                    <div class="flex flex-col items-center">
                        <span class="text-xs text-muted-foreground mb-1">Days</span>
                        <input type="text" x-model="days" maxlength="2" @input="days = $event.target.value.replace(/\D/g, '').slice(0,2).padStart(2, '0')" class="w-12 h-10 text-center rounded-md border border-input bg-transparent text-lg font-mono">
                    </div>
                    <span class="text-xl font-bold mt-4">:</span>
                    <div class="flex flex-col items-center">
                        <span class="text-xs text-muted-foreground mb-1">Hours</span>
                        <input type="text" x-model="hours" maxlength="2" @input="hours = Math.min(23, $event.target.value.replace(/\D/g, '')).toString().padStart(2, '0')" class="w-12 h-10 text-center rounded-md border border-input bg-transparent text-lg font-mono">
                    </div>
                </div>
            </div>
        "#,

        "file-upload" => r#"
            <div class="flex flex-col items-center justify-center w-full max-w-sm rounded-lg border-2 border-dashed border-input p-6 hover:bg-accent/50 cursor-pointer">
                <svg class="h-10 w-10 text-muted-foreground mb-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
                <p class="text-sm text-muted-foreground">Drag and drop or click to upload</p>
            </div>
        "#,

        "flash-messages" => r#"
            <div class="w-full max-w-sm space-y-2">
                <div class="flex items-center gap-2 rounded-lg border border-green-500/50 bg-green-50 dark:bg-green-950 p-3">
                    <svg class="h-4 w-4 text-green-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                    <span class="text-sm text-green-900 dark:text-green-100">Changes saved successfully!</span>
                </div>
            </div>
        "#,

        "formatted-number" => r#"
            <div class="space-y-2 text-center">
                <div class="text-2xl font-bold">$1,234.56</div>
                <div class="text-sm text-muted-foreground">Formatted currency</div>
            </div>
        "#,

        "hover-card" => r##"
            <div x-data="{
                open: false,
                pos: { x: 0, y: 0 },
                updatePos() {
                    const rect = this.$refs.trigger.getBoundingClientRect();
                    this.pos.x = rect.left + rect.width / 2;
                    this.pos.y = rect.bottom + 8;
                }
            }" class="inline-block">
                <a href="#" x-ref="trigger" @mouseenter="updatePos(); open = true" @mouseleave="open = false" class="text-sm underline">@nextjs</a>
                <template x-teleport="body">
                    <div x-show="open" x-cloak
                         :style="`left: ${pos.x}px; top: ${pos.y}px; transform: translateX(-50%);`"
                         class="fixed w-64 rounded-md border bg-popover text-popover-foreground p-4 shadow-md z-[9999]"
                         @mouseenter="open = true" @mouseleave="open = false">
                        <div class="flex gap-4">
                            <div class="h-12 w-12 rounded-full bg-muted"></div>
                            <div class="space-y-1">
                                <h4 class="text-sm font-semibold">Next.js</h4>
                                <p class="text-xs text-muted-foreground">The React Framework for the Web</p>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        "##,

        "input-otp" => r#"
            <div x-data="{
                otp: ['', '', '', '', '', ''],
                focusNext(index) {
                    if (index < 5) this.$refs['otp' + (index + 1)].focus();
                },
                focusPrev(index) {
                    if (index > 0) this.$refs['otp' + (index - 1)].focus();
                },
                handleInput(index, event) {
                    const val = event.target.value.replace(/\D/g, '');
                    if (val.length > 0) {
                        this.otp[index] = val[0];
                        event.target.value = val[0];
                        this.focusNext(index);
                    }
                },
                handleKeydown(index, event) {
                    if (event.key === 'Backspace') {
                        if (this.otp[index] === '') {
                            this.focusPrev(index);
                        } else {
                            this.otp[index] = '';
                        }
                    } else if (event.key === 'ArrowLeft') {
                        this.focusPrev(index);
                    } else if (event.key === 'ArrowRight') {
                        this.focusNext(index);
                    }
                },
                handlePaste(event) {
                    event.preventDefault();
                    const paste = (event.clipboardData || window.clipboardData).getData('text').replace(/\D/g, '').slice(0, 6);
                    for (let i = 0; i < paste.length && i < 6; i++) {
                        this.otp[i] = paste[i];
                    }
                    if (paste.length > 0) {
                        const focusIdx = Math.min(paste.length, 5);
                        this.$refs['otp' + focusIdx].focus();
                    }
                }
            }" class="flex gap-2" @paste="handlePaste($event)">
                <input x-ref="otp0" type="text" inputmode="numeric" maxlength="1" :value="otp[0]" @input="handleInput(0, $event)" @keydown="handleKeydown(0, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
                <input x-ref="otp1" type="text" inputmode="numeric" maxlength="1" :value="otp[1]" @input="handleInput(1, $event)" @keydown="handleKeydown(1, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
                <input x-ref="otp2" type="text" inputmode="numeric" maxlength="1" :value="otp[2]" @input="handleInput(2, $event)" @keydown="handleKeydown(2, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
                <span class="flex items-center text-muted-foreground text-xl">-</span>
                <input x-ref="otp3" type="text" inputmode="numeric" maxlength="1" :value="otp[3]" @input="handleInput(3, $event)" @keydown="handleKeydown(3, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
                <input x-ref="otp4" type="text" inputmode="numeric" maxlength="1" :value="otp[4]" @input="handleInput(4, $event)" @keydown="handleKeydown(4, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
                <input x-ref="otp5" type="text" inputmode="numeric" maxlength="1" :value="otp[5]" @input="handleInput(5, $event)" @keydown="handleKeydown(5, $event)" class="w-12 h-12 text-center rounded-md border border-input bg-transparent text-xl font-semibold focus:border-primary focus:ring-1 focus:ring-primary">
            </div>
        "#,

        "menubar" => r#"
            <div x-data="{ activeMenu: null }" class="flex h-10 items-center space-x-1 rounded-md border bg-background p-1">
                <div class="relative">
                    <button @click="activeMenu = activeMenu === 'file' ? null : 'file'" @mouseenter="activeMenu && (activeMenu = 'file')" :class="activeMenu === 'file' ? 'bg-accent' : ''" class="px-3 py-1.5 text-sm font-medium rounded-sm hover:bg-accent transition-colors">File</button>
                    <template x-teleport="body">
                        <div x-show="activeMenu === 'file'" @click.away="activeMenu = null" x-cloak class="fixed z-50 min-w-[180px] rounded-md border bg-popover p-1 text-popover-foreground shadow-md" :style="`left: ${$el.previousElementSibling?.getBoundingClientRect().left}px; top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`" x-init="$watch('activeMenu', () => { if (activeMenu === 'file') { const rect = $refs.fileBtn?.getBoundingClientRect(); if (rect) { $el.style.left = rect.left + 'px'; $el.style.top = (rect.bottom + 4) + 'px'; } } })">
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">New Tab <span class="float-right text-muted-foreground text-xs">⌘T</span></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">New Window <span class="float-right text-muted-foreground text-xs">⌘N</span></div>
                            <div class="h-px bg-border my-1"></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Share</div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Print <span class="float-right text-muted-foreground text-xs">⌘P</span></div>
                        </div>
                    </template>
                </div>
                <div class="relative">
                    <button @click="activeMenu = activeMenu === 'edit' ? null : 'edit'" @mouseenter="activeMenu && (activeMenu = 'edit')" :class="activeMenu === 'edit' ? 'bg-accent' : ''" class="px-3 py-1.5 text-sm font-medium rounded-sm hover:bg-accent transition-colors">Edit</button>
                    <template x-teleport="body">
                        <div x-show="activeMenu === 'edit'" @click.away="activeMenu = null" x-cloak class="fixed z-50 min-w-[180px] rounded-md border bg-popover p-1 text-popover-foreground shadow-md" :style="`left: ${$el.previousElementSibling?.getBoundingClientRect().left}px; top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`">
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Undo <span class="float-right text-muted-foreground text-xs">⌘Z</span></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Redo <span class="float-right text-muted-foreground text-xs">⇧⌘Z</span></div>
                            <div class="h-px bg-border my-1"></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Cut <span class="float-right text-muted-foreground text-xs">⌘X</span></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Copy <span class="float-right text-muted-foreground text-xs">⌘C</span></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Paste <span class="float-right text-muted-foreground text-xs">⌘V</span></div>
                        </div>
                    </template>
                </div>
                <div class="relative">
                    <button @click="activeMenu = activeMenu === 'view' ? null : 'view'" @mouseenter="activeMenu && (activeMenu = 'view')" :class="activeMenu === 'view' ? 'bg-accent' : ''" class="px-3 py-1.5 text-sm font-medium rounded-sm hover:bg-accent transition-colors">View</button>
                    <template x-teleport="body">
                        <div x-show="activeMenu === 'view'" @click.away="activeMenu = null" x-cloak class="fixed z-50 min-w-[180px] rounded-md border bg-popover p-1 text-popover-foreground shadow-md" :style="`left: ${$el.previousElementSibling?.getBoundingClientRect().left}px; top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`">
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer flex items-center gap-2"><svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6 9 17l-5-5"/></svg> Always Show Bookmarks Bar</div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Always Show Full URLs</div>
                            <div class="h-px bg-border my-1"></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Reload <span class="float-right text-muted-foreground text-xs">⌘R</span></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer opacity-50 cursor-not-allowed">Force Reload</div>
                            <div class="h-px bg-border my-1"></div>
                            <div class="px-2 py-1.5 text-sm rounded-sm hover:bg-accent cursor-pointer">Toggle Fullscreen</div>
                        </div>
                    </template>
                </div>
                <button class="px-3 py-1.5 text-sm font-medium rounded-sm hover:bg-accent transition-colors">Help</button>
            </div>
        "#,

        "minmax-editor" => r#"
            <div x-data="{ expanded: false }" class="w-full max-w-xs">
                <button @click="expanded = !expanded" class="flex w-full items-center justify-between rounded-md border px-3 py-2 text-sm">
                    <span>Price Range</span>
                    <span class="text-muted-foreground">$10 - $100</span>
                </button>
                <div x-show="expanded" x-collapse class="mt-2 space-y-2 rounded-md border p-3">
                    <div class="flex items-center gap-2">
                        <span class="text-sm w-10">Min</span>
                        <input type="number" value="10" class="flex-1 h-8 rounded-md border border-input bg-transparent px-2 text-sm">
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-sm w-10">Max</span>
                        <input type="number" value="100" class="flex-1 h-8 rounded-md border border-input bg-transparent px-2 text-sm">
                    </div>
                </div>
            </div>
        "#,

        "navigation-menu" => r##"
            <nav x-data="{ activeNav: null }" class="relative flex items-center gap-1">
                <div class="relative">
                    <button @mouseenter="activeNav = 'getting-started'" @mouseleave="activeNav = null" class="group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none">
                        Getting Started
                        <svg class="relative top-[1px] ml-1 h-3 w-3 transition duration-200" :class="activeNav === 'getting-started' ? 'rotate-180' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="activeNav === 'getting-started'" @mouseenter="activeNav = 'getting-started'" @mouseleave="activeNav = null" x-cloak class="fixed left-1/2 z-50 w-[400px] -translate-x-1/2 rounded-md border bg-popover p-4 text-popover-foreground shadow-lg" :style="`top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`" x-transition:enter="transition ease-out duration-200" x-transition:enter-start="opacity-0 translate-y-1" x-transition:enter-end="opacity-100 translate-y-0">
                            <div class="grid gap-3 md:grid-cols-2">
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Introduction</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">Re-usable components built using Radix UI and Tailwind CSS.</p>
                                </a>
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Installation</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">How to install dependencies and structure your app.</p>
                                </a>
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Typography</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">Styles for headings, paragraphs, lists...etc</p>
                                </a>
                            </div>
                        </div>
                    </template>
                </div>
                <div class="relative">
                    <button @mouseenter="activeNav = 'components'" @mouseleave="activeNav = null" class="group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none">
                        Components
                        <svg class="relative top-[1px] ml-1 h-3 w-3 transition duration-200" :class="activeNav === 'components' ? 'rotate-180' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="activeNav === 'components'" @mouseenter="activeNav = 'components'" @mouseleave="activeNav = null" x-cloak class="fixed left-1/2 z-50 w-[500px] -translate-x-1/2 rounded-md border bg-popover p-4 text-popover-foreground shadow-lg" :style="`top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`" x-transition:enter="transition ease-out duration-200" x-transition:enter-start="opacity-0 translate-y-1" x-transition:enter-end="opacity-100 translate-y-0">
                            <div class="grid gap-3 md:grid-cols-2">
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Alert Dialog</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">A modal dialog that interrupts the user.</p>
                                </a>
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Hover Card</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">For sighted users to preview content.</p>
                                </a>
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Progress</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">Displays progress visually.</p>
                                </a>
                                <a href="#" class="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div class="text-sm font-medium leading-none">Scroll-area</div>
                                    <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">Augments native scroll functionality.</p>
                                </a>
                            </div>
                        </div>
                    </template>
                </div>
                <a href="#" class="group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none">
                    Documentation
                </a>
            </nav>
        "##,

        "numeric-input" => r#"
            <div class="flex items-center">
                <button class="h-9 w-9 rounded-l-md border border-r-0 hover:bg-accent">-</button>
                <input type="number" value="5" class="h-9 w-16 border text-center bg-transparent text-sm">
                <button class="h-9 w-9 rounded-r-md border border-l-0 hover:bg-accent">+</button>
            </div>
        "#,

        "pagination" => r#"
            <nav class="flex items-center space-x-2">
                <button class="h-9 px-3 rounded-md border text-sm hover:bg-accent flex items-center gap-1">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
                    Previous
                </button>
                <button class="h-9 w-9 rounded-md border bg-primary text-primary-foreground text-sm">1</button>
                <button class="h-9 w-9 rounded-md border text-sm hover:bg-accent">2</button>
                <button class="h-9 w-9 rounded-md border text-sm hover:bg-accent">3</button>
                <span class="h-9 w-9 flex items-center justify-center text-muted-foreground">...</span>
                <button class="h-9 w-9 rounded-md border text-sm hover:bg-accent">10</button>
                <button class="h-9 px-3 rounded-md border text-sm hover:bg-accent flex items-center gap-1">
                    Next
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
                </button>
            </nav>
        "#,

        "password-input" => r#"
            <div x-data="{ show: false, password: 'mypassword123' }" class="relative w-full max-w-xs">
                <input :type="show ? 'text' : 'password'" x-model="password" placeholder="Enter password" class="flex h-10 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm pr-10 focus:outline-none focus:ring-1 focus:ring-primary">
                <button type="button" @click="show = !show" class="absolute right-0 top-0 h-10 w-10 flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors" :aria-label="show ? 'Hide password' : 'Show password'">
                    <svg x-show="!show" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
                    <svg x-show="show" x-cloak width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/><path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
                </button>
                <p class="mt-2 text-xs text-muted-foreground">Click the eye icon to <span x-text="show ? 'hide' : 'show'"></span> password</p>
            </div>
        "#,

        "resizable" => r#"
            <div class="flex h-[150px] w-full max-w-md rounded-lg border">
                <div class="flex-1 p-4 flex items-center justify-center">
                    <span class="text-sm text-muted-foreground">Panel 1</span>
                </div>
                <div class="w-1 bg-border cursor-col-resize hover:bg-primary/50"></div>
                <div class="flex-1 p-4 flex items-center justify-center">
                    <span class="text-sm text-muted-foreground">Panel 2</span>
                </div>
            </div>
        "#,

        "scroll-area" => r#"
            <div class="h-[150px] w-[200px] rounded-md border overflow-hidden">
                <div class="p-4 space-y-4 h-full overflow-y-auto">
                    <div class="text-sm">Item 1</div>
                    <div class="text-sm">Item 2</div>
                    <div class="text-sm">Item 3</div>
                    <div class="text-sm">Item 4</div>
                    <div class="text-sm">Item 5</div>
                    <div class="text-sm">Item 6</div>
                    <div class="text-sm">Item 7</div>
                    <div class="text-sm">Item 8</div>
                </div>
            </div>
        "#,

        "searchable-select" => r#"
            <div class="relative w-[200px]">
                <div class="flex h-9 items-center rounded-md border border-input px-3">
                    <svg class="h-4 w-4 mr-2 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                    <input type="text" placeholder="Search..." class="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground">
                </div>
            </div>
        "#,

        "sheet" => r#"
            <div x-data="{ open: false }">
                <button @click="open = true" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Open Sheet</button>
                <template x-teleport="body">
                    <div x-show="open" x-cloak class="fixed inset-0 z-50">
                        <div class="fixed inset-0 bg-black/80" @click="open = false"></div>
                        <div class="fixed inset-y-0 right-0 z-50 w-3/4 max-w-sm border-l bg-background p-6 shadow-lg">
                            <h3 class="text-lg font-semibold">Sheet Title</h3>
                            <p class="text-sm text-muted-foreground mt-2">Sheet content slides in from the side.</p>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "time-picker" => r#"
            <div class="flex items-center gap-2">
                <select class="h-9 rounded-md border border-input bg-transparent px-2 text-sm">
                    <option>09</option><option>10</option><option>11</option><option>12</option>
                </select>
                <span>:</span>
                <select class="h-9 rounded-md border border-input bg-transparent px-2 text-sm">
                    <option>00</option><option>15</option><option>30</option><option>45</option>
                </select>
                <select class="h-9 rounded-md border border-input bg-transparent px-2 text-sm">
                    <option>AM</option><option>PM</option>
                </select>
            </div>
        "#,

        "toast" => r#"
            <div class="flex flex-col items-center gap-4">
                <div class="flex gap-2">
                    <button @click="$dispatch('toast', { title: 'Success!', description: 'Your changes have been saved.', variant: 'success', duration: 3000 })" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-green-600 text-white hover:bg-green-700">Success</button>
                    <button @click="$dispatch('toast', { title: 'Error', description: 'Something went wrong.', variant: 'error', duration: 3000 })" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90">Error</button>
                    <button @click="$dispatch('toast', { title: 'Info', description: 'Here is some information.', variant: 'default', duration: 3000 })" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90">Info</button>
                </div>
                <div id="toast-container-preview" class="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none" x-data="{ toasts: [] }" @toast.window="const id = Date.now(); toasts.push({ id, ...$event.detail }); setTimeout(() => { toasts = toasts.filter(t => t.id !== id) }, $event.detail.duration || 3000)">
                    <template x-for="toast in toasts" :key="toast.id">
                        <div class="pointer-events-auto w-80 rounded-lg border p-4 shadow-lg transition-all duration-300" :class="{ 'border-green-500/50 bg-green-50 dark:bg-green-950 text-green-900 dark:text-green-100': toast.variant === 'success', 'border-destructive/50 bg-red-50 dark:bg-red-950 text-red-900 dark:text-red-100': toast.variant === 'error', 'border-border bg-background': toast.variant === 'default' }" x-transition:enter="transform ease-out duration-300" x-transition:enter-start="translate-x-full opacity-0" x-transition:enter-end="translate-x-0 opacity-100" x-transition:leave="transform ease-in duration-200" x-transition:leave-start="translate-x-0 opacity-100" x-transition:leave-end="translate-x-full opacity-0">
                            <div class="flex items-start gap-3">
                                <template x-if="toast.variant === 'success'"><svg class="h-5 w-5 text-green-600 dark:text-green-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg></template>
                                <template x-if="toast.variant === 'error'"><svg class="h-5 w-5 text-red-600 dark:text-red-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg></template>
                                <div class="flex-1"><p class="text-sm font-semibold" x-text="toast.title"></p><p class="mt-1 text-sm opacity-90" x-text="toast.description"></p></div>
                                <button @click="toasts = toasts.filter(t => t.id !== toast.id)" class="shrink-0 rounded p-1 opacity-70 hover:opacity-100"><svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
                            </div>
                        </div>
                    </template>
                </div>
            </div>
        "#,

        "toggle-group" => r#"
            <div class="inline-flex rounded-md border">
                <button class="h-9 px-3 text-sm rounded-l-md bg-accent">Left</button>
                <button class="h-9 px-3 text-sm border-l hover:bg-accent">Center</button>
                <button class="h-9 px-3 text-sm border-l rounded-r-md hover:bg-accent">Right</button>
            </div>
        "#,

        "validation-errors" => r#"
            <div class="w-full max-w-sm rounded-md border border-destructive/50 bg-destructive/10 p-4">
                <div class="flex items-center gap-2 text-destructive mb-2">
                    <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                    <span class="text-sm font-medium">Please fix the following errors:</span>
                </div>
                <ul class="list-disc list-inside text-sm text-destructive space-y-1">
                    <li>Email is required</li>
                    <li>Password must be at least 8 characters</li>
                </ul>
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

// Primary button (default)
&lt;@button&gt;Click me&lt;/@&gt;

// Variants
&lt;@button variant=button::Variant::Secondary&gt;Secondary&lt;/@&gt;
&lt;@button variant=button::Variant::Destructive&gt;Delete&lt;/@&gt;
&lt;@button variant=button::Variant::Outline&gt;Outline&lt;/@&gt;
&lt;@button variant=button::Variant::Ghost&gt;Ghost&lt;/@&gt;

// Sizes
&lt;@button size=button::Size::Sm&gt;Small&lt;/@&gt;
&lt;@button size=button::Size::Lg&gt;Large&lt;/@&gt;

// Disabled
&lt;@button disabled=true&gt;Disabled&lt;/@&gt;"#.to_string(),

        "input" => r#"@import components/input.wtz as input

// Basic input
&lt;@input name="email" placeholder=Some("Enter email") /&gt;

// With type
&lt;@input name="password" type_attr="password" /&gt;

// Disabled
&lt;@input name="readonly" disabled=true /&gt;"#.to_string(),

        "card" => r#"@import components/card.wtz as card

&lt;@card&gt;
    &lt;@card::header&gt;
        &lt;@card::title&gt;Card Title&lt;/@&gt;
        &lt;@card::description&gt;Card description&lt;/@&gt;
    &lt;/@&gt;
    &lt;@card::content&gt;
        Your content here
    &lt;/@&gt;
    &lt;@card::footer&gt;
        &lt;@button&gt;Action&lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "checkbox" => r#"@import components/checkbox.wtz as checkbox

// Basic checkbox
&lt;@checkbox name="terms" /&gt;

// With label
&lt;div class="flex items-center gap-2"&gt;
    &lt;@checkbox id="accept" name="accept" /&gt;
    &lt;@label for_id="accept"&gt;Accept terms&lt;/@&gt;
&lt;/div&gt;

// Checked by default
&lt;@checkbox name="newsletter" checked=true /&gt;"#.to_string(),

        "switch" => r#"@import components/switch.wtz as switch

// Basic switch
&lt;@switch name="notifications" /&gt;

// With label
&lt;div class="flex items-center gap-2"&gt;
    &lt;@switch id="airplane" name="airplane" /&gt;
    &lt;@label for_id="airplane"&gt;Airplane Mode&lt;/@&gt;
&lt;/div&gt;

// Enabled by default
&lt;@switch name="wifi" checked=true /&gt;"#.to_string(),

        "badge" => r#"@import components/badge.wtz as badge

// Default badge
&lt;@badge&gt;Badge&lt;/@&gt;

// Variants
&lt;@badge variant=badge::Variant::Secondary&gt;Secondary&lt;/@&gt;
&lt;@badge variant=badge::Variant::Destructive&gt;Error&lt;/@&gt;
&lt;@badge variant=badge::Variant::Outline&gt;Outline&lt;/@&gt;"#.to_string(),

        "alert" => r#"@import components/alert.wtz as alert

// Default alert
&lt;@alert&gt;
    &lt;@alert::title&gt;Heads up!&lt;/@&gt;
    &lt;@alert::description&gt;
        You can add components using the CLI.
    &lt;/@&gt;
&lt;/@&gt;

// Destructive alert
&lt;@alert variant=alert::Variant::Destructive&gt;
    &lt;@alert::title&gt;Error&lt;/@&gt;
    &lt;@alert::description&gt;Something went wrong.&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "avatar" => r#"@import components/avatar.wtz as avatar

// With image
&lt;@avatar src="https://github.com/user.png" alt="User" /&gt;

// With fallback initials
&lt;@avatar fallback="JD" /&gt;

// Different sizes
&lt;@avatar src="..." size=avatar::Size::Sm /&gt;
&lt;@avatar src="..." size=avatar::Size::Lg /&gt;"#.to_string(),

        "progress" => r#"@import components/progress.wtz as progress

// Basic progress bar
&lt;@progress value=60 /&gt;

// With max value
&lt;@progress value=30 max=50 /&gt;

// Indeterminate (no value)
&lt;@progress /&gt;"#.to_string(),

        "skeleton" => r#"@import components/skeleton.wtz as skeleton

// Basic skeleton
&lt;@skeleton class="h-4 w-[200px]" /&gt;

// Card skeleton
&lt;div class="flex items-center space-x-4"&gt;
    &lt;@skeleton class="h-12 w-12 rounded-full" /&gt;
    &lt;div class="space-y-2"&gt;
        &lt;@skeleton class="h-4 w-[200px]" /&gt;
        &lt;@skeleton class="h-4 w-[150px]" /&gt;
    &lt;/div&gt;
&lt;/div&gt;"#.to_string(),

        "separator" => r#"@import components/separator.wtz as separator

// Horizontal (default)
&lt;@separator /&gt;

// Vertical
&lt;@separator orientation=separator::Orientation::Vertical /&gt;

// With custom class
&lt;@separator class="my-4" /&gt;"#.to_string(),

        "label" => r#"@import components/label.wtz as label

// Basic label
&lt;@label&gt;Email&lt;/@&gt;

// Associated with input
&lt;@label for_id="email"&gt;Email&lt;/@&gt;
&lt;@input id="email" name="email" /&gt;"#.to_string(),

        "textarea" => r#"@import components/textarea.wtz as textarea

// Basic textarea
&lt;@textarea
    name="message"
    placeholder=Some("Type your message...")
/&gt;

// With rows
&lt;@textarea name="bio" rows=5 /&gt;"#.to_string(),

        "select" => r#"@import components/select.wtz as select

&lt;@select name="fruit" placeholder="Select a fruit"&gt;
    &lt;@select::item value="apple"&gt;Apple&lt;/@&gt;
    &lt;@select::item value="banana"&gt;Banana&lt;/@&gt;
    &lt;@select::item value="orange"&gt;Orange&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "tabs" => r#"@import components/tabs.wtz as tabs

&lt;@tabs default="account"&gt;
    &lt;@tabs::list&gt;
        &lt;@tabs::trigger value="account"&gt;Account&lt;/@&gt;
        &lt;@tabs::trigger value="password"&gt;Password&lt;/@&gt;
    &lt;/@&gt;
    &lt;@tabs::content value="account"&gt;
        Account settings here
    &lt;/@&gt;
    &lt;@tabs::content value="password"&gt;
        Password settings here
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "accordion" => r#"@import components/accordion.wtz as accordion

&lt;@accordion&gt;
    &lt;@accordion::item value="item-1"&gt;
        &lt;@accordion::trigger&gt;Is it accessible?&lt;/@&gt;
        &lt;@accordion::content&gt;
            Yes. It adheres to the WAI-ARIA pattern.
        &lt;/@&gt;
    &lt;/@&gt;
    &lt;@accordion::item value="item-2"&gt;
        &lt;@accordion::trigger&gt;Is it styled?&lt;/@&gt;
        &lt;@accordion::content&gt;
            Yes. It comes with default styles.
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "dialog" => r#"@import components/dialog.wtz as dialog

&lt;@dialog&gt;
    &lt;@dialog::trigger&gt;
        &lt;@button&gt;Open Dialog&lt;/@&gt;
    &lt;/@&gt;
    &lt;@dialog::content&gt;
        &lt;@dialog::header&gt;
            &lt;@dialog::title&gt;Edit profile&lt;/@&gt;
            &lt;@dialog::description&gt;
                Make changes to your profile here.
            &lt;/@&gt;
        &lt;/@&gt;
        &lt;@dialog::footer&gt;
            &lt;@button variant=button::Variant::Outline&gt;Cancel&lt;/@&gt;
            &lt;@button&gt;Save&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "alert-dialog" => r#"@import components/alert_dialog.wtz as alert_dialog

&lt;@alert_dialog&gt;
    &lt;@alert_dialog::trigger&gt;
        &lt;@button variant=button::Variant::Destructive&gt;
            Delete Account
        &lt;/@&gt;
    &lt;/@&gt;
    &lt;@alert_dialog::content&gt;
        &lt;@alert_dialog::header&gt;
            &lt;@alert_dialog::title&gt;Are you sure?&lt;/@&gt;
            &lt;@alert_dialog::description&gt;
                This action cannot be undone.
            &lt;/@&gt;
        &lt;/@&gt;
        &lt;@alert_dialog::footer&gt;
            &lt;@alert_dialog::cancel&gt;Cancel&lt;/@&gt;
            &lt;@alert_dialog::action&gt;Delete&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "dropdown" => r#"@import components/dropdown.wtz as dropdown

&lt;@dropdown&gt;
    &lt;@dropdown::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Open Menu&lt;/@&gt;
    &lt;/@&gt;
    &lt;@dropdown::content&gt;
        &lt;@dropdown::label&gt;My Account&lt;/@&gt;
        &lt;@dropdown::separator /&gt;
        &lt;@dropdown::item&gt;Profile&lt;/@&gt;
        &lt;@dropdown::item&gt;Settings&lt;/@&gt;
        &lt;@dropdown::separator /&gt;
        &lt;@dropdown::item class="text-destructive"&gt;Log out&lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "popover" => r#"@import components/popover.wtz as popover

&lt;@popover&gt;
    &lt;@popover::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Open&lt;/@&gt;
    &lt;/@&gt;
    &lt;@popover::content&gt;
        &lt;div class="grid gap-4"&gt;
            &lt;h4 class="font-medium"&gt;Dimensions&lt;/h4&gt;
            &lt;p class="text-sm text-muted-foreground"&gt;
                Set the dimensions for the layer.
            &lt;/p&gt;
        &lt;/div&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "tooltip" => r#"@import components/tooltip.wtz as tooltip

&lt;@tooltip&gt;
    &lt;@tooltip::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Hover me&lt;/@&gt;
    &lt;/@&gt;
    &lt;@tooltip::content&gt;
        Add to library
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "slider" => r#"@import components/slider.wtz as slider

// Basic slider
&lt;@slider name="volume" value=50 /&gt;

// With min/max
&lt;@slider name="price" min=0 max=1000 value=250 /&gt;

// With step
&lt;@slider name="rating" min=1 max=5 step=1 value=3 /&gt;"#.to_string(),

        "radio-group" => r#"@import components/radio_group.wtz as radio_group

&lt;@radio_group name="size" default="default"&gt;
    &lt;div class="flex items-center space-x-2"&gt;
        &lt;@radio_group::item value="default" id="r1" /&gt;
        &lt;@label for_id="r1"&gt;Default&lt;/@&gt;
    &lt;/div&gt;
    &lt;div class="flex items-center space-x-2"&gt;
        &lt;@radio_group::item value="comfortable" id="r2" /&gt;
        &lt;@label for_id="r2"&gt;Comfortable&lt;/@&gt;
    &lt;/div&gt;
    &lt;div class="flex items-center space-x-2"&gt;
        &lt;@radio_group::item value="compact" id="r3" /&gt;
        &lt;@label for_id="r3"&gt;Compact&lt;/@&gt;
    &lt;/div&gt;
&lt;/@&gt;"#.to_string(),

        "toggle" => r#"@import components/toggle.wtz as toggle

// Basic toggle
&lt;@toggle&gt;
    &lt;svg ...&gt;Bold icon&lt;/svg&gt;
&lt;/@&gt;

// With text
&lt;@toggle&gt;Bold&lt;/@&gt;

// Pressed by default
&lt;@toggle pressed=true&gt;On&lt;/@&gt;"#.to_string(),

        "table" => r#"@import components/table.wtz as table

&lt;@table&gt;
    &lt;@table::header&gt;
        &lt;@table::row&gt;
            &lt;@table::head&gt;Invoice&lt;/@&gt;
            &lt;@table::head&gt;Status&lt;/@&gt;
            &lt;@table::head class="text-right"&gt;Amount&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
    &lt;@table::body&gt;
        @for invoice in invoices {
            &lt;@table::row&gt;
                &lt;@table::cell&gt;@invoice.id&lt;/@&gt;
                &lt;@table::cell&gt;@invoice.status&lt;/@&gt;
                &lt;@table::cell class="text-right"&gt;
                    @invoice.amount
                &lt;/@&gt;
            &lt;/@&gt;
        }
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "breadcrumb" => r##"@import components/breadcrumb.wtz as breadcrumb

&lt;@breadcrumb&gt;
    &lt;@breadcrumb::list&gt;
        &lt;@breadcrumb::item&gt;
            &lt;@breadcrumb::link href="/"&gt;Home&lt;/@&gt;
        &lt;/@&gt;
        &lt;@breadcrumb::separator /&gt;
        &lt;@breadcrumb::item&gt;
            &lt;@breadcrumb::link href="/components"&gt;Components&lt;/@&gt;
        &lt;/@&gt;
        &lt;@breadcrumb::separator /&gt;
        &lt;@breadcrumb::item&gt;
            &lt;@breadcrumb::page&gt;Breadcrumb&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"##.to_string(),

        "form" => r#"@import components/form.wtz as form

&lt;@form action="/submit" method="post"&gt;
    &lt;@form::field&gt;
        &lt;@form::label&gt;Username&lt;/@&gt;
        &lt;@form::control&gt;
            &lt;@input name="username" /&gt;
        &lt;/@&gt;
        &lt;@form::description&gt;
            This is your public display name.
        &lt;/@&gt;
    &lt;/@&gt;
    &lt;@button type="submit"&gt;Submit&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "collapsible" => r#"@import components/collapsible.wtz as collapsible

&lt;@collapsible&gt;
    &lt;div class="flex items-center justify-between"&gt;
        &lt;h4&gt;@peduarte starred 3 repositories&lt;/h4&gt;
        &lt;@collapsible::trigger&gt;
            &lt;@button variant=button::Variant::Ghost size=button::Size::Sm&gt;
                Toggle
            &lt;/@&gt;
        &lt;/@&gt;
    &lt;/div&gt;
    &lt;@collapsible::content&gt;
        &lt;div class="space-y-2"&gt;
            &lt;div&gt;@radix-ui/colors&lt;/div&gt;
            &lt;div&gt;@stitches/react&lt;/div&gt;
        &lt;/div&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "combobox" => r#"@import components/combobox.wtz as combobox

&lt;@combobox name="framework" placeholder="Select framework..."&gt;
    &lt;@combobox::item value="next"&gt;Next.js&lt;/@&gt;
    &lt;@combobox::item value="svelte"&gt;SvelteKit&lt;/@&gt;
    &lt;@combobox::item value="nuxt"&gt;Nuxt&lt;/@&gt;
    &lt;@combobox::item value="remix"&gt;Remix&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "ajax-select" => r#"@import components/ajax_select.wtz as ajax_select

// Ajax select with remote search
&lt;@ajax_select
    name="user"
    placeholder="Search users..."
    url="/api/users/search"
    label_key="name"
    value_key="id"
/&gt;"#.to_string(),

        "aspect-ratio" => r#"@import components/aspect_ratio.wtz as aspect_ratio

// 16:9 aspect ratio
&lt;@aspect_ratio ratio="16/9"&gt;
    &lt;img src="/image.jpg" class="object-cover" /&gt;
&lt;/@&gt;

// Square
&lt;@aspect_ratio ratio="1/1"&gt;
    &lt;img src="/avatar.jpg" /&gt;
&lt;/@&gt;"#.to_string(),

        "calendar" => r#"@import components/calendar.wtz as calendar

// Basic calendar
&lt;@calendar name="date" /&gt;

// With selected date
&lt;@calendar name="date" selected=Some("2026-01-15") /&gt;

// With min/max dates
&lt;@calendar
    name="booking"
    min_date=Some("2026-01-01")
    max_date=Some("2026-12-31")
/&gt;"#.to_string(),

        "carousel" => r#"@import components/carousel.wtz as carousel

// Basic carousel
&lt;@carousel&gt;
    &lt;@carousel::item&gt;&lt;img src="/slide1.jpg" /&gt;&lt;/@&gt;
    &lt;@carousel::item&gt;&lt;img src="/slide2.jpg" /&gt;&lt;/@&gt;
    &lt;@carousel::item&gt;&lt;img src="/slide3.jpg" /&gt;&lt;/@&gt;
&lt;/@&gt;

// Auto-playing carousel
&lt;@carousel::autoplay interval=5000&gt;
    &lt;@carousel::item&gt;Slide 1&lt;/@&gt;
    &lt;@carousel::item&gt;Slide 2&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "command" => r#"@import components/command.wtz as command

&lt;@command&gt;
    &lt;@command::input placeholder="Type a command..." /&gt;
    &lt;@command::list&gt;
        &lt;@command::group heading="Suggestions"&gt;
            &lt;@command::item&gt;Calendar&lt;/@&gt;
            &lt;@command::item&gt;Calculator&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "context-menu" => r#"@import components/context_menu.wtz as context_menu

&lt;@context_menu&gt;
    &lt;@context_menu::trigger&gt;
        &lt;div class="border border-dashed p-8"&gt;
            Right click here
        &lt;/div&gt;
    &lt;/@&gt;
    &lt;@context_menu::content&gt;
        &lt;@context_menu::item&gt;Cut&lt;/@&gt;
        &lt;@context_menu::item&gt;Copy&lt;/@&gt;
        &lt;@context_menu::item&gt;Paste&lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "date-picker" => r#"@import components/date_picker.wtz as date_picker

// Basic date picker
&lt;@date_picker name="date" placeholder="Pick a date" /&gt;

// With preselected value
&lt;@date_picker name="birthday" value=Some("2000-01-15") /&gt;

// With format
&lt;@date_picker name="date" format="DD/MM/YYYY" /&gt;"#.to_string(),

        "datetime-picker" => r#"@import components/datetime_picker.wtz as datetime_picker

// Combined date and time picker
&lt;@datetime_picker name="event_time" /&gt;

// With default value
&lt;@datetime_picker
    name="meeting"
    value=Some("2026-01-15T14:30")
/&gt;"#.to_string(),

        "drawer" => r#"@import components/drawer.wtz as drawer

&lt;@drawer
    title="Edit Profile"
    description=Some("Make changes to your profile.")
    trigger=@{ &lt;@button&gt;Open Drawer&lt;/@&gt; }
    content=@{
        &lt;p&gt;Drawer content here&lt;/p&gt;
    }
    footer=Some(@{
        &lt;@button&gt;Save changes&lt;/@&gt;
    })
/&gt;"#.to_string(),

        "duration-input" => r#"@import components/duration_input.wtz as duration_input

// Duration with hours/minutes/days
&lt;@duration_input
    name="timeout"
    value=2
    unit="hours"
/&gt;

// Custom units
&lt;@duration_input
    name="delay"
    units=vec!["seconds", "minutes", "hours"]
/&gt;"#.to_string(),

        "file-upload" => r#"@import components/file_upload.wtz as file_upload

// Basic file upload
&lt;@file_upload name="document" /&gt;

// With accepted types
&lt;@file_upload
    name="image"
    accept=".jpg,.png,.gif"
    max_size_mb=5
/&gt;

// Multiple files
&lt;@file_upload name="files" multiple=true /&gt;"#.to_string(),

        "flash-messages" => r#"@import components/flash_messages.wtz as flash

// Place container in layout (once)
&lt;@flash::container /&gt;

// Show messages
&lt;@flash::success message="Saved successfully!" /&gt;
&lt;@flash::error message="Something went wrong." /&gt;
&lt;@flash::warning message="Please review your input." /&gt;"#.to_string(),

        "formatted-number" => r#"@import components/formatted_number.wtz as formatted_number

// Currency formatting
&lt;@formatted_number value=1234.56 style="currency" currency="USD" /&gt;

// Percentage
&lt;@formatted_number value=0.85 style="percent" /&gt;

// With locale
&lt;@formatted_number value=1234567 locale="de-DE" /&gt;"#.to_string(),

        "hover-card" => r#"@import components/hover_card.wtz as hover_card

&lt;@hover_card
    trigger=@{ &lt;a href="/user/jane"&gt;@jane&lt;/a&gt; }
    content=@{
        &lt;div class="flex gap-4"&gt;
            &lt;img src="/avatar.jpg" class="h-12 w-12 rounded-full" /&gt;
            &lt;div&gt;
                &lt;h4 class="font-semibold"&gt;Jane Doe&lt;/h4&gt;
                &lt;p class="text-sm text-muted-foreground"&gt;Software Engineer&lt;/p&gt;
            &lt;/div&gt;
        &lt;/div&gt;
    }
/&gt;"#.to_string(),

        "input-otp" => r#"@import components/input_otp.wtz as input_otp

// 6-digit OTP
&lt;@input_otp name="code" length=6 /&gt;

// With separator
&lt;@input_otp name="code" length=6 separator_after=3 /&gt;

// 4-digit PIN
&lt;@input_otp name="pin" length=4 /&gt;"#.to_string(),

        "menubar" => r#"@import components/menubar.wtz as menubar

&lt;@menubar&gt;
    &lt;@menubar::menu&gt;
        &lt;@menubar::trigger&gt;File&lt;/@&gt;
        &lt;@menubar::content&gt;
            &lt;@menubar::item&gt;New&lt;/@&gt;
            &lt;@menubar::item&gt;Open&lt;/@&gt;
            &lt;@menubar::separator /&gt;
            &lt;@menubar::item&gt;Exit&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
    &lt;@menubar::menu&gt;
        &lt;@menubar::trigger&gt;Edit&lt;/@&gt;
        &lt;@menubar::content&gt;
            &lt;@menubar::item&gt;Undo&lt;/@&gt;
            &lt;@menubar::item&gt;Redo&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "minmax-editor" => r#"@import components/minmax_editor.wtz as minmax_editor

// Price range editor
&lt;@minmax_editor
    name="price"
    label="Price Range"
    min_value=0
    max_value=100
    prefix="$"
/&gt;

// With step
&lt;@minmax_editor
    name="quantity"
    label="Quantity"
    min_value=1
    max_value=1000
    step=10
/&gt;"#.to_string(),

        "navigation-menu" => r#"@import components/navigation_menu.wtz as nav

&lt;@nav&gt;
    &lt;@nav::list&gt;
        &lt;@nav::item&gt;
            &lt;@nav::link href="/getting-started"&gt;
                Getting Started
            &lt;/@&gt;
        &lt;/@&gt;
        &lt;@nav::item&gt;
            &lt;@nav::trigger&gt;Components&lt;/@&gt;
            &lt;@nav::content&gt;
                &lt;@nav::link href="/components/button"&gt;Button&lt;/@&gt;
                &lt;@nav::link href="/components/input"&gt;Input&lt;/@&gt;
            &lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "numeric-input" => r#"@import components/numeric_input.wtz as numeric_input

// Basic numeric input with +/- buttons
&lt;@numeric_input name="quantity" value=1 min=0 max=100 /&gt;

// With step
&lt;@numeric_input name="price" value=10 step=0.5 /&gt;"#.to_string(),

        "pagination" => r#"@import components/pagination.wtz as pagination

// Full pagination with page numbers
&lt;@pagination
    current=3
    total=10
    siblings=1
    href_template="/posts?page={page}"
/&gt;

// Simple prev/next
&lt;@pagination::simple
    current=2
    total=5
    prev_url=Some("/posts?page=1")
    next_url=Some("/posts?page=3")
/&gt;"#.to_string(),

        "password-input" => r#"@import components/password_input.wtz as password_input

// Password input with show/hide toggle
&lt;@password_input name="password" placeholder="Enter password" /&gt;

// With strength indicator
&lt;@password_input
    name="new_password"
    show_strength=true
/&gt;"#.to_string(),

        "resizable" => r#"@import components/resizable.wtz as resizable

&lt;@resizable direction="horizontal"&gt;
    &lt;@resizable::panel default_size=50&gt;
        Panel 1
    &lt;/@&gt;
    &lt;@resizable::handle /&gt;
    &lt;@resizable::panel default_size=50&gt;
        Panel 2
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "scroll-area" => r#"@import components/scroll_area.wtz as scroll_area

&lt;@scroll_area class="h-[200px]"&gt;
    @for item in items {
        &lt;div&gt;@item&lt;/div&gt;
    }
&lt;/@&gt;

// Horizontal scroll
&lt;@scroll_area orientation="horizontal"&gt;
    &lt;div class="flex gap-4"&gt;...&lt;/div&gt;
&lt;/@&gt;"#.to_string(),

        "searchable-select" => r#"@import components/searchable_select.wtz as searchable_select

&lt;@searchable_select name="country" placeholder="Search countries..."&gt;
    &lt;@searchable_select::item value="us"&gt;United States&lt;/@&gt;
    &lt;@searchable_select::item value="uk"&gt;United Kingdom&lt;/@&gt;
    &lt;@searchable_select::item value="ca"&gt;Canada&lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "sheet" => r#"@import components/sheet.wtz as sheet

&lt;@sheet side=sheet::Side::Right&gt;
    &lt;@sheet::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Open&lt;/@&gt;
    &lt;/@&gt;
    &lt;@sheet::content&gt;
        &lt;@sheet::header&gt;
            &lt;@sheet::title&gt;Edit profile&lt;/@&gt;
            &lt;@sheet::description&gt;
                Make changes to your profile here.
            &lt;/@&gt;
        &lt;/@&gt;
        &lt;div class="py-4"&gt;Content&lt;/div&gt;
        &lt;@sheet::footer&gt;
            &lt;@button&gt;Save&lt;/@&gt;
        &lt;/@&gt;
    &lt;/@&gt;
&lt;/@&gt;"#.to_string(),

        "time-picker" => r#"@import components/time_picker.wtz as time_picker

// 12-hour format
&lt;@time_picker name="time" format="12h" /&gt;

// 24-hour format
&lt;@time_picker name="time" format="24h" /&gt;

// With default value
&lt;@time_picker name="meeting" value="14:30" /&gt;"#.to_string(),

        "toast" => r#"@import components/toast.wtz as toast

// Place container once in layout
&lt;@toast::container position="bottom-right" /&gt;

// Show toasts via Alpine.js
&lt;button @click="$dispatch('toast', {
    title: 'Success',
    description: 'Your changes were saved.',
    variant: 'success'
})"&gt;Save&lt;/button&gt;

// Or use shorthand functions
&lt;@toast::success title="Saved!" description=Some("Changes applied.") /&gt;
&lt;@toast::error title="Error" description=Some("Something went wrong.") /&gt;"#.to_string(),

        "toggle-group" => r#"@import components/toggle_group.wtz as toggle_group

@let items = vec![
    ToggleItem { value: "left".into(), label: "Left".into(), ... },
    ToggleItem { value: "center".into(), label: "Center".into(), ... },
    ToggleItem { value: "right".into(), label: "Right".into(), ... },
];

// Single selection
&lt;@toggle_group::single name="align" items=&items selected=Some("left") /&gt;

// Multiple selection
&lt;@toggle_group::multiple name="format" items=&items selected=vec!["left"] /&gt;"#.to_string(),

        "validation-errors" => r#"@import components/validation_errors.wtz as validation_errors

// Display validation errors
&lt;@validation_errors errors=@errors /&gt;

// With custom title
&lt;@validation_errors
    errors=@errors
    title="Please fix the following:"
/&gt;"#.to_string(),

        _ => {
            let comp_name = component.replace('-', "_");
            format!(
                r#"@import components/{}.wtz as {}

&lt;@{}&gt;
    Content here
&lt;/@&gt;"#,
                component, comp_name, comp_name
            )
        }
    }
}
