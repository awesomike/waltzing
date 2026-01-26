//! Waltzing Showcase Server
//!
//! A web server that showcases all discovered Waltzing template libraries.
//! Libraries are auto-discovered from the `libraries/` directory at compile time.

use axum::{
    extract::Path,
    http::HeaderMap,
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
        .route("/library/{id}/layout/{layout}", get(layout_showcase))
        .route("/library/{id}/block/{block}", get(block_showcase))
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

        /* Remove shadows from cards in preview areas */
        .component-preview .shadow,
        .component-preview .shadow-sm,
        .component-preview .shadow-md,
        .component-preview .shadow-lg,
        .component-preview [class*="shadow"] {
            box-shadow: none !important;
        }
    </style>
    <script src="https://unpkg.com/htmx.org@2.0.4"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/focus@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/collapse@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/persist@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
"#
}

/// Check if the request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Generate just the main content area for HTMX partial updates
fn main_content_partial(title: &str, content: &str, item_type: &str) -> String {
    format!(
        r##"<header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
    <div class="flex items-center gap-4">
        <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="4" x2="20" y1="12" y2="12"></line>
                <line x1="4" x2="20" y1="6" y2="6"></line>
                <line x1="4" x2="20" y1="18" y2="18"></line>
            </svg>
        </button>
        <div>
            <p class="text-xs text-muted-foreground uppercase tracking-wide">{item_type}</p>
            <h2 class="text-xl font-semibold">{title}</h2>
        </div>
    </div>
    {toggle}
</header>
<div class="p-6 max-w-4xl">
    {content}
</div>"##,
        title = title,
        item_type = item_type,
        toggle = theme_toggle(),
        content = content
    )
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
            let layouts = get_layout_list(&id);
            let blocks = get_block_list(&id);
            let sidebar = generate_sidebar(&id, &components, &layouts, &blocks, None);
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
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: $persist(true).as('sidebar_open') }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card transition-all duration-300 flex flex-col h-full overflow-hidden"
        >
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
async fn component_showcase(
    headers: HeaderMap,
    Path((id, component)): Path<(String, String)>,
) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let content = generate_component_detail(&component);
            let title = id_to_title(&component);

            // For HTMX requests, return only the main content
            if is_htmx_request(&headers) {
                return Html(main_content_partial(&title, &content, "Component"));
            }

            // Full page for regular requests
            let components = get_component_list(&id);
            let layouts = get_layout_list(&id);
            let blocks = get_block_list(&id);
            let sidebar = generate_sidebar(&id, &components, &layouts, &blocks, Some(&component));

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - {lib_name} - Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: $persist(true).as('sidebar_open') }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card transition-all duration-300 flex flex-col h-full overflow-hidden"
        >
            {sidebar}
        </aside>

        <!-- Main content -->
        <main id="main-content" class="flex-1 overflow-y-auto">
            <header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div class="flex items-center gap-4">
                    <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="4" x2="20" y1="12" y2="12"></line>
                            <line x1="4" x2="20" y1="6" y2="6"></line>
                            <line x1="4" x2="20" y1="18" y2="18"></line>
                        </svg>
                    </button>
                    <div>
                        <p class="text-xs text-muted-foreground uppercase tracking-wide">Component</p>
                        <h2 class="text-xl font-semibold">{title}</h2>
                    </div>
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
                sidebar = sidebar,
                toggle = theme_toggle(),
                content = content
            );

            Html(html)
        }
        None => Html(not_found_page(&id)),
    }
}

async fn layout_showcase(
    headers: HeaderMap,
    Path((id, layout)): Path<(String, String)>,
) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let content = generate_layout_detail(&layout);
            let title = id_to_title(&layout);

            // For HTMX requests, return only the main content
            if is_htmx_request(&headers) {
                return Html(main_content_partial(&title, &content, "Layout"));
            }

            // Full page for regular requests
            let components = get_component_list(&id);
            let layouts = get_layout_list(&id);
            let blocks = get_block_list(&id);
            let sidebar = generate_sidebar(&id, &components, &layouts, &blocks, Some(&layout));

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} Layout - {lib_name} - Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: $persist(true).as('sidebar_open') }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card transition-all duration-300 flex flex-col h-full overflow-hidden"
        >
            {sidebar}
        </aside>

        <!-- Main content -->
        <main id="main-content" class="flex-1 overflow-y-auto">
            <header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div class="flex items-center gap-4">
                    <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="4" x2="20" y1="12" y2="12"></line>
                            <line x1="4" x2="20" y1="6" y2="6"></line>
                            <line x1="4" x2="20" y1="18" y2="18"></line>
                        </svg>
                    </button>
                    <div>
                        <p class="text-xs text-muted-foreground uppercase tracking-wide">Layout</p>
                        <h2 class="text-xl font-semibold">{title}</h2>
                    </div>
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
                title = title,
                lib_name = lib.name,
                head = head_common(),
                sidebar = sidebar,
                toggle = theme_toggle(),
                content = content
            );

            Html(html)
        }
        None => Html(not_found_page(&id)),
    }
}

async fn block_showcase(
    headers: HeaderMap,
    Path((id, block)): Path<(String, String)>,
) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let content = generate_block_detail(&block);
            let title = id_to_title(&block);

            // For HTMX requests, return only the main content
            if is_htmx_request(&headers) {
                return Html(main_content_partial(&title, &content, "Block"));
            }

            // Full page for regular requests
            let components = get_component_list(&id);
            let layouts = get_layout_list(&id);
            let blocks = get_block_list(&id);
            let sidebar = generate_sidebar(&id, &components, &layouts, &blocks, Some(&block));

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} Block - {lib_name} - Waltzing Showcase</title>
    {head}
</head>
<body class="min-h-screen" x-data="{{ dark: true, sidebarOpen: $persist(true).as('sidebar_open') }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="flex h-screen overflow-hidden">
        <!-- Sidebar -->
        <aside
            :class="sidebarOpen ? 'w-64' : 'w-0'"
            class="flex-shrink-0 border-r border-border bg-card transition-all duration-300 flex flex-col h-full overflow-hidden"
        >
            {sidebar}
        </aside>

        <!-- Main content -->
        <main id="main-content" class="flex-1 overflow-y-auto">
            <header class="sticky top-0 z-10 flex items-center justify-between p-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div class="flex items-center gap-4">
                    <button @click="sidebarOpen = !sidebarOpen" class="p-2 rounded-md hover:bg-accent">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="4" x2="20" y1="12" y2="12"></line>
                            <line x1="4" x2="20" y1="6" y2="6"></line>
                            <line x1="4" x2="20" y1="18" y2="18"></line>
                        </svg>
                    </button>
                    <div>
                        <p class="text-xs text-muted-foreground uppercase tracking-wide">Block</p>
                        <h2 class="text-xl font-semibold">{title}</h2>
                    </div>
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
                title = title,
                lib_name = lib.name,
                head = head_common(),
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

/// Convert kebab-case id to Title Case display name
fn id_to_title(id: &str) -> String {
    id.split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_component_list(library_id: &str) -> Vec<(String, String)> {
    match library_id {
        "waltzing-ui" => generated::waltzing_ui::COMPONENTS
            .iter()
            .map(|id| (id.to_string(), id_to_title(id)))
            .collect(),
        _ => vec![],
    }
}

fn get_layout_list(library_id: &str) -> Vec<(String, String)> {
    match library_id {
        "waltzing-ui" => generated::waltzing_ui::LAYOUTS
            .iter()
            .map(|id| (id.to_string(), id_to_title(id)))
            .collect(),
        _ => vec![],
    }
}

fn get_block_list(library_id: &str) -> Vec<(String, String)> {
    match library_id {
        "waltzing-ui" => generated::waltzing_ui::BLOCKS
            .iter()
            .map(|id| (id.to_string(), id_to_title(id)))
            .collect(),
        _ => vec![],
    }
}

fn generate_sidebar(
    library_id: &str,
    components: &[(String, String)],
    layouts: &[(String, String)],
    blocks: &[(String, String)],
    active: Option<&str>,
) -> String {
    // Icons for sections
    let components_icon = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/></svg>"#;
    let layouts_icon = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>"#;
    let blocks_icon = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/></svg>"#;

    // Generate nav items for each section
    fn generate_nav_items(
        items: &[(String, String)],
        library_id: &str,
        item_type: &str,
        active: Option<&str>,
    ) -> String {
        items
            .iter()
            .map(|(id, name)| {
                let is_active = active == Some(id.as_str());
                let active_class = if is_active {
                    "bg-accent text-accent-foreground"
                } else {
                    "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                };
                let url = format!("/library/{}/{}/{}", library_id, item_type, id);
                format!(
                    r##"<a href="{url}" hx-get="{url}" hx-target="#main-content" hx-push-url="true" class="block px-4 py-1.5 text-sm rounded-md transition-colors {cls}">{name}</a>"##,
                    url = url,
                    cls = active_class,
                    name = name
                )
            })
            .collect()
    }

    // Generate a collapsible section
    fn generate_section(
        title: &str,
        items_html: &str,
        section_key: &str,
        icon: &str,
        is_empty: bool,
    ) -> String {
        if is_empty {
            return String::new();
        }
        format!(
            r##"<div x-data="{{ open: $persist(true).as('sidebar_{key}') }}" class="mb-2">
                <button @click="open = !open" class="flex items-center justify-between w-full px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
                    <span class="flex items-center gap-2">
                        {icon}
                        {title}
                    </span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="transition-transform" :class="open ? 'rotate-90' : ''">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>
                <div x-show="open" x-collapse class="space-y-0.5 mt-1">
                    {items}
                </div>
            </div>"##,
            key = section_key,
            icon = icon,
            title = title,
            items = items_html
        )
    }

    let component_items = generate_nav_items(components, library_id, "component", active);
    let layout_items = generate_nav_items(layouts, library_id, "layout", active);
    let block_items = generate_nav_items(blocks, library_id, "block", active);

    let components_section = generate_section(
        "Components",
        &component_items,
        &format!("{}_components", library_id),
        components_icon,
        components.is_empty(),
    );
    let layouts_section = generate_section(
        "Layouts",
        &layout_items,
        &format!("{}_layouts", library_id),
        layouts_icon,
        layouts.is_empty(),
    );
    let blocks_section = generate_section(
        "Blocks",
        &block_items,
        &format!("{}_blocks", library_id),
        blocks_icon,
        blocks.is_empty(),
    );

    format!(
        r##"<div class="h-full flex flex-col">
            <!-- Header with back button, library info, and collapse toggle -->
            <div class="p-4 border-b border-border">
                <div class="flex items-center justify-between mb-3">
                    <a href="/" class="flex items-center gap-2 text-muted-foreground hover:text-foreground">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="m12 19-7-7 7-7"/>
                            <path d="M19 12H5"/>
                        </svg>
                        <span class="text-sm">Back</span>
                    </a>
                    <!-- Collapse toggle button -->
                    <button @click="sidebarOpen = false" class="p-1.5 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors" title="Collapse sidebar">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <rect width="18" height="18" x="3" y="3" rx="2"/>
                            <path d="M9 3v18"/>
                        </svg>
                    </button>
                </div>
                <h1 class="text-lg font-semibold">waltzing-ui</h1>
                <p class="text-sm text-muted-foreground">v0.1.0</p>
            </div>

            <!-- Navigation sections -->
            <nav class="flex-1 overflow-y-auto py-2">
                {components_section}
                {layouts_section}
                {blocks_section}
            </nav>
        </div>"##,
        components_section = components_section,
        layouts_section = layouts_section,
        blocks_section = blocks_section
    )
}

fn generate_all_components_preview(library_id: &str, components: &[(String, String)]) -> String {
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

fn read_template_file(category: &str, name: &str) -> Option<String> {
    let path = format!("libraries/waltzing-ui/{}/{}.wtz", category, name);
    std::fs::read_to_string(&path).ok()
}

fn extract_doc_comment(content: &str) -> String {
    let mut in_comment = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@*") {
            in_comment = true;
            let after = trimmed.trim_start_matches("@*").trim();
            if !after.is_empty() && !after.starts_with("*") {
                lines.push(after.to_string());
            }
            continue;
        }
        if in_comment {
            // Stop at end of comment
            if trimmed.ends_with("*@") || trimmed == "*@" {
                break;
            }
            let cleaned = trimmed.trim_start_matches("*").trim();
            // Stop at @example, @param, or other documentation tags
            if cleaned.starts_with("@example")
                || cleaned.starts_with("@param")
                || cleaned.starts_with("@import")
            {
                break;
            }
            // Skip lines starting with @ (other tags)
            if !cleaned.starts_with("@") && !cleaned.is_empty() {
                lines.push(cleaned.to_string());
            }
        }
    }

    lines.join(" ")
}

fn html_escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

fn generate_layout_detail(layout: &str) -> String {
    let source = read_template_file("layouts", layout).unwrap_or_default();
    let description = if !source.is_empty() {
        extract_doc_comment(&source)
    } else {
        get_layout_description(layout).to_string()
    };
    let preview = get_layout_preview(layout);
    let usage = get_layout_usage(layout);

    format!(
        r##"
        <div class="space-y-8">
            <!-- Description -->
            <section>
                <div class="rounded-lg border border-border p-6 bg-card">
                    <p class="text-muted-foreground">{description}</p>
                </div>
            </section>

            <!-- Live Preview -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Preview</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="bg-background min-h-[500px] relative">
                        {preview}
                    </div>
                </div>
            </section>

            <!-- Usage -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Usage</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="p-4 bg-muted/30">
                        <pre class="text-sm overflow-x-auto"><code class="language-waltzing">{usage}</code></pre>
                    </div>
                </div>
            </section>

            <!-- Source -->
            <section x-data="{{ open: false }}">
                <button @click="open = !open" class="flex items-center gap-2 text-lg font-semibold mb-4 hover:text-primary">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="open ? 'rotate-90' : ''" class="transition-transform">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                    Source Code
                </button>
                <div x-show="open" x-collapse class="rounded-lg border border-border overflow-hidden">
                    <div class="p-4 bg-muted/30 max-h-[600px] overflow-auto">
                        <pre class="text-sm"><code>{source}</code></pre>
                    </div>
                </div>
            </section>
        </div>
        "##,
        description = description,
        preview = preview,
        usage = usage,
        source = html_escape(&source)
    )
}

fn get_layout_description(layout: &str) -> &'static str {
    match layout {
        "base" => "The base layout provides the foundation for all pages, including HTML document structure, meta tags, theme support, and common scripts/styles.",
        "auth" => "Authentication layout optimized for login, signup, and password reset pages. Centers content with optional branding areas.",
        "dashboard" => "Dashboard layout with a collapsible sidebar, header with user menu, and main content area. Ideal for admin panels and data-heavy applications.",
        "marketing" => "Marketing layout with navigation header, hero sections support, and footer. Perfect for landing pages and marketing sites.",
        "settings" => "Settings layout with a sidebar navigation for different setting categories. Great for user preferences and configuration pages.",
        "sidebar" => "Sidebar layout with a persistent navigation sidebar and main content area. Suitable for documentation sites and multi-section applications.",
        _ => "A layout template for structuring page content.",
    }
}

fn get_layout_preview(layout: &str) -> &'static str {
    match layout {
        "base" => r##"
            <div class="h-[500px] flex flex-col">
                <!-- Simulated browser chrome -->
                <div class="bg-muted/50 border-b px-4 py-2 flex items-center gap-2">
                    <div class="flex gap-1.5">
                        <div class="w-3 h-3 rounded-full bg-red-500/70"></div>
                        <div class="w-3 h-3 rounded-full bg-yellow-500/70"></div>
                        <div class="w-3 h-3 rounded-full bg-green-500/70"></div>
                    </div>
                    <div class="flex-1 mx-4">
                        <div class="bg-background rounded px-3 py-1 text-xs text-muted-foreground">localhost:3000</div>
                    </div>
                </div>
                <!-- Page content -->
                <div class="flex-1 p-8">
                    <h1 class="text-3xl font-bold mb-4">Welcome to My App</h1>
                    <p class="text-muted-foreground mb-6">The base layout provides the HTML document structure, theme support, and script loading.</p>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="rounded-lg border bg-card p-4">
                            <h3 class="font-semibold mb-2">Feature One</h3>
                            <p class="text-sm text-muted-foreground">Content goes here</p>
                        </div>
                        <div class="rounded-lg border bg-card p-4">
                            <h3 class="font-semibold mb-2">Feature Two</h3>
                            <p class="text-sm text-muted-foreground">More content here</p>
                        </div>
                    </div>
                </div>
            </div>
        "##,
        "auth" => r##"
            <div class="flex h-[500px]">
                <!-- Brand side -->
                <div class="hidden md:flex md:w-1/2 bg-muted p-10 flex-col">
                    <div class="flex items-center gap-2 text-lg font-semibold">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-primary"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                        <span>Acme Inc</span>
                    </div>
                    <div class="flex flex-1 items-center justify-center">
                        <blockquote class="space-y-2 max-w-lg">
                            <p class="text-lg">"This library has saved me countless hours of work and helped me deliver stunning designs to my clients faster than ever before."</p>
                            <footer class="text-sm text-muted-foreground">Sofia Davis, CEO</footer>
                        </blockquote>
                    </div>
                </div>
                <!-- Auth form side -->
                <div class="flex-1 flex items-center justify-center p-6 lg:p-10">
                    <div class="w-full max-w-md space-y-6">
                        <div class="rounded-xl border bg-card text-card-foreground shadow p-6 space-y-6">
                            <div class="space-y-2 text-center">
                                <h1 class="text-2xl font-semibold tracking-tight">Welcome back</h1>
                                <p class="text-sm text-muted-foreground">Enter your email to sign in to your account</p>
                            </div>
                            <div class="space-y-4">
                                <div class="space-y-2">
                                    <label class="text-sm font-medium">Email</label>
                                    <input type="email" placeholder="name@example.com" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                </div>
                                <div class="space-y-2">
                                    <label class="text-sm font-medium">Password</label>
                                    <input type="password" placeholder="••••••••" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                </div>
                                <button class="inline-flex w-full items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Sign In</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        "##,
        "dashboard" => r##"
            <div x-data="{ sidebarOpen: true }" class="flex h-[500px]">
                <!-- Sidebar -->
                <aside :class="sidebarOpen ? 'w-64' : 'w-16'" class="border-r bg-card flex flex-col transition-all duration-300">
                    <div class="flex h-14 items-center border-b px-4">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-primary"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                        <span x-show="sidebarOpen" class="ml-2 font-semibold">Dashboard</span>
                    </div>
                    <nav class="flex-1 p-2 space-y-1">
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium bg-accent text-accent-foreground">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
                            <span x-show="sidebarOpen">Home</span>
                        </a>
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                            <span x-show="sidebarOpen">Users</span>
                        </a>
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
                            <span x-show="sidebarOpen">Settings</span>
                        </a>
                    </nav>
                </aside>
                <!-- Main content -->
                <div class="flex-1 flex flex-col overflow-hidden">
                    <header class="h-14 border-b flex items-center justify-between px-4">
                        <div class="flex items-center gap-4">
                            <button @click="sidebarOpen = !sidebarOpen" class="p-2 hover:bg-accent rounded-md">
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
                            </button>
                            <span class="font-medium">Overview</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <div class="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center text-sm font-medium">JD</div>
                        </div>
                    </header>
                    <main class="flex-1 overflow-auto p-6">
                        <div class="grid grid-cols-3 gap-4 mb-6">
                            <div class="rounded-lg border bg-card p-4">
                                <p class="text-sm text-muted-foreground">Total Revenue</p>
                                <p class="text-2xl font-bold">$45,231.89</p>
                                <p class="text-xs text-green-500">+20.1% from last month</p>
                            </div>
                            <div class="rounded-lg border bg-card p-4">
                                <p class="text-sm text-muted-foreground">Subscriptions</p>
                                <p class="text-2xl font-bold">+2350</p>
                                <p class="text-xs text-green-500">+180.1% from last month</p>
                            </div>
                            <div class="rounded-lg border bg-card p-4">
                                <p class="text-sm text-muted-foreground">Active Now</p>
                                <p class="text-2xl font-bold">+573</p>
                                <p class="text-xs text-muted-foreground">+201 since last hour</p>
                            </div>
                        </div>
                    </main>
                </div>
            </div>
        "##,
        "marketing" => r##"
            <div class="flex flex-col h-[500px]">
                <!-- Header -->
                <header class="border-b">
                    <div class="flex h-14 items-center justify-between px-6">
                        <div class="flex items-center gap-6">
                            <a href="#" class="flex items-center gap-2 font-semibold">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-primary"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                                <span>Acme Inc</span>
                            </a>
                            <nav class="hidden md:flex gap-6">
                                <a href="#" class="text-sm font-medium text-muted-foreground hover:text-foreground">Features</a>
                                <a href="#" class="text-sm font-medium text-muted-foreground hover:text-foreground">Pricing</a>
                                <a href="#" class="text-sm font-medium text-muted-foreground hover:text-foreground">About</a>
                            </nav>
                        </div>
                        <div class="flex items-center gap-4">
                            <a href="#" class="text-sm font-medium">Log in</a>
                            <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 bg-primary text-primary-foreground shadow hover:bg-primary/90">Get Started</button>
                        </div>
                    </div>
                </header>
                <!-- Hero -->
                <div class="flex-1 flex items-center justify-center p-8">
                    <div class="text-center max-w-3xl space-y-6">
                        <h1 class="text-4xl font-bold tracking-tight sm:text-5xl">Build something amazing today</h1>
                        <p class="text-xl text-muted-foreground">Create beautiful, responsive websites with our component library. Ship faster, build better.</p>
                        <div class="flex gap-4 justify-center">
                            <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-11 px-8 bg-primary text-primary-foreground shadow hover:bg-primary/90">Get Started</button>
                            <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-11 px-8 border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground">Learn More</button>
                        </div>
                    </div>
                </div>
                <!-- Footer -->
                <footer class="border-t py-4 px-6">
                    <p class="text-center text-sm text-muted-foreground">© 2024 Acme Inc. All rights reserved.</p>
                </footer>
            </div>
        "##,
        "settings" => r##"
            <div class="flex h-[500px]">
                <!-- Settings sidebar -->
                <aside class="w-56 border-r p-4">
                    <h2 class="text-lg font-semibold mb-4">Settings</h2>
                    <nav class="space-y-1">
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium bg-accent text-accent-foreground">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                            Profile
                        </a>
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/></svg>
                            Account
                        </a>
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                            Security
                        </a>
                        <a href="#" class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/></svg>
                            Notifications
                        </a>
                    </nav>
                </aside>
                <!-- Settings content -->
                <div class="flex-1 p-6 overflow-auto">
                    <div class="max-w-2xl">
                        <h1 class="text-2xl font-bold mb-2">Profile</h1>
                        <p class="text-muted-foreground mb-6">Manage your profile settings and preferences.</p>
                        <div class="space-y-6">
                            <div class="space-y-2">
                                <label class="text-sm font-medium">Display Name</label>
                                <input type="text" value="John Doe" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">Email</label>
                                <input type="email" value="john@example.com" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                            </div>
                            <div class="space-y-2">
                                <label class="text-sm font-medium">Bio</label>
                                <textarea rows="3" class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Tell us about yourself"></textarea>
                            </div>
                            <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 bg-primary text-primary-foreground shadow hover:bg-primary/90">Save Changes</button>
                        </div>
                    </div>
                </div>
            </div>
        "##,
        "sidebar" => r##"
            <div class="flex h-[500px]">
                <!-- Sidebar navigation -->
                <aside class="w-64 border-r bg-card flex flex-col">
                    <div class="flex h-14 items-center border-b px-4">
                        <span class="font-semibold">Documentation</span>
                    </div>
                    <nav class="flex-1 p-4 space-y-1 overflow-auto">
                        <div class="mb-4">
                            <h3 class="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">Getting Started</h3>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm bg-accent text-accent-foreground">Introduction</a>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Installation</a>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Quick Start</a>
                        </div>
                        <div class="mb-4">
                            <h3 class="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">Components</h3>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Button</a>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Card</a>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Dialog</a>
                            <a href="#" class="flex items-center rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent/50">Input</a>
                        </div>
                    </nav>
                </aside>
                <!-- Documentation content -->
                <div class="flex-1 p-8 overflow-auto">
                    <article class="prose prose-invert max-w-none">
                        <h1 class="text-3xl font-bold mb-4">Introduction</h1>
                        <p class="text-muted-foreground mb-4">Welcome to the documentation. This guide will help you get started with building beautiful user interfaces.</p>
                        <h2 class="text-xl font-semibold mb-2 mt-6">Overview</h2>
                        <p class="text-muted-foreground mb-4">Our component library provides a comprehensive set of accessible, reusable components that you can use to build modern web applications.</p>
                        <div class="rounded-lg border bg-muted/30 p-4 mt-6">
                            <p class="text-sm font-medium mb-2">Quick Tip</p>
                            <p class="text-sm text-muted-foreground">Check out the Installation guide to get started with your first project.</p>
                        </div>
                    </article>
                </div>
            </div>
        "##,
        _ => r##"
            <div class="p-8 flex flex-col items-center justify-center h-full min-h-[300px] text-center">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted-foreground mb-4">
                    <rect width="18" height="18" x="3" y="3" rx="2"/>
                    <path d="M3 9h18"/>
                    <path d="M9 21V9"/>
                </svg>
                <h4 class="text-lg font-medium mb-2">Layout Preview</h4>
                <p class="text-sm text-muted-foreground">Preview not available for this layout.</p>
            </div>
        "##,
    }
}

fn get_layout_usage(layout: &str) -> &'static str {
    match layout {
        "base" => r#"@import /layouts/base.wtz as base

<@base::apply title="My Page">
    <h1>Welcome</h1>
    <p>Your content here...</p>
</@>"#,
        "auth" => r#"@import /layouts/auth.wtz as auth

<@auth::apply title="Login">
    <@auth::card>
        <form>
            <!-- Login form fields -->
        </form>
    </@>
</@>"#,
        "dashboard" => r#"@import /layouts/dashboard.wtz as dashboard

<@dashboard::apply
    title="Dashboard"
    user_name="John Doe"
    user_email="john@example.com"
>
    <div class="grid gap-4">
        <!-- Dashboard content -->
    </div>
</@>"#,
        "marketing" => r#"@import /layouts/marketing.wtz as marketing

<@marketing::apply title="Welcome">
    <@marketing::hero
        title="Build Something Amazing"
        description="Get started with our platform"
    />
    <@marketing::features />
</@>"#,
        "settings" => r#"@import /layouts/settings.wtz as settings

<@settings::apply
    title="Settings"
    active_section="profile"
>
    <!-- Settings content for active section -->
</@>"#,
        "sidebar" => r#"@import /layouts/sidebar.wtz as sidebar

@let nav_items = vec![
    sidebar::NavItem { label: "Home", href: "/", active: true },
    sidebar::NavItem { label: "About", href: "/about", active: false },
]

<@sidebar::apply title="Docs" nav_items=@nav_items>
    <article>
        <!-- Page content -->
    </article>
</@>"#,
        _ => r#"@import /layouts/LAYOUT_NAME.wtz as layout

<@layout::apply title="Page Title">
    <!-- Your content here -->
</@>"#,
    }
}

fn get_block_category_and_file(block: &str) -> (&'static str, String) {
    match block {
        "login" => ("auth", "login".to_string()),
        "login-01" | "login-02" | "login-03" | "login-04" | "login-05" => ("auth", block.to_string()),
        "signup" => ("auth", "signup".to_string()),
        "signup-01" => ("auth", block.to_string()),
        "otp-01" => ("auth", block.to_string()),
        "confirm-dialog" => ("dialogs", "confirm".to_string()),
        "delete-dialog" => ("dialogs", "delete".to_string()),
        "share-dialog" => ("dialogs", "share".to_string()),
        "contact-form" => ("forms", "contact".to_string()),
        "profile-form" => ("forms", "profile".to_string()),
        "sidebar-01" | "sidebar-02" | "sidebar-03" | "sidebar-07" | "sidebar-11" | "sidebar-12" | "sidebar-15" => ("sidebar", block.to_string()),
        "dashboard-01" => ("dashboard", block.to_string()),
        "calendar-01" => ("calendar", block.to_string()),
        _ => ("", block.to_string()),
    }
}

fn read_block_source(block: &str) -> Option<String> {
    let (category, file) = get_block_category_and_file(block);
    if category.is_empty() {
        return None;
    }
    let path = format!("libraries/waltzing-ui/blocks/{}/{}.wtz", category, file);
    std::fs::read_to_string(&path).ok()
}

fn generate_block_detail(block: &str) -> String {
    let source = read_block_source(block).unwrap_or_default();
    let description = if !source.is_empty() {
        let doc = extract_doc_comment(&source);
        if doc.is_empty() {
            get_block_description(block).to_string()
        } else {
            doc
        }
    } else {
        get_block_description(block).to_string()
    };
    let preview = get_block_preview(block);
    let usage = get_block_usage(block);

    format!(
        r##"
        <div class="space-y-8">
            <!-- Description -->
            <section>
                <div class="rounded-lg border border-border p-6 bg-card">
                    <p class="text-muted-foreground">{description}</p>
                </div>
            </section>

            <!-- Preview -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Preview</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="p-8 bg-card/50 flex items-center justify-center min-h-[300px]">
                        {preview}
                    </div>
                </div>
            </section>

            <!-- Usage -->
            <section>
                <h3 class="text-lg font-semibold mb-4">Usage</h3>
                <div class="rounded-lg border border-border overflow-hidden">
                    <div class="p-4 bg-muted/30">
                        <pre class="text-sm overflow-x-auto"><code class="language-waltzing">{usage}</code></pre>
                    </div>
                </div>
            </section>

            <!-- Source -->
            <section x-data="{{ open: false }}">
                <button @click="open = !open" class="flex items-center gap-2 text-lg font-semibold mb-4 hover:text-primary">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="open ? 'rotate-90' : ''" class="transition-transform">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                    Source Code
                </button>
                <div x-show="open" x-collapse class="rounded-lg border border-border overflow-hidden">
                    <div class="p-4 bg-muted/30 max-h-[600px] overflow-auto">
                        <pre class="text-sm"><code>{source}</code></pre>
                    </div>
                </div>
            </section>
        </div>
        "##,
        description = description,
        preview = preview,
        usage = usage,
        source = html_escape(&source)
    )
}

fn get_block_description(block: &str) -> &'static str {
    match block {
        "login" | "login-01" | "login-02" | "login-03" | "login-04" | "login-05" =>
            "A complete login form block with email/password fields, remember me option, and forgot password link. Ready to integrate with your authentication backend.",
        "signup" | "signup-01" =>
            "A signup form block with name, email, password fields, and terms acceptance. Includes validation states and social signup options.",
        "otp-01" =>
            "One-time password verification block with a 6-digit code input. Perfect for two-factor authentication flows.",
        "sidebar-01" | "sidebar-02" | "sidebar-03" =>
            "A navigation sidebar block with collapsible sections, icons, and active state highlighting.",
        "sidebar-07" =>
            "A sidebar with user profile section, navigation items, and workspace switcher.",
        "sidebar-11" =>
            "A file explorer sidebar with collapsible folder tree, file type icons, and search functionality.",
        "sidebar-12" =>
            "A calendar sidebar with mini calendar, upcoming events list, and date selection.",
        "sidebar-15" =>
            "A settings sidebar with categorized navigation and notification badges.",
        "dashboard-01" =>
            "A complete dashboard block with stats cards, recent activity table, and quick actions.",
        "calendar-01" =>
            "A full calendar block with month view, event display, and navigation controls.",
        "confirm-dialog" =>
            "A confirmation dialog for actions that require user acknowledgment.",
        "delete-dialog" =>
            "A destructive action dialog with warning styling for delete confirmations.",
        "share-dialog" =>
            "A share dialog with link copying, email sharing, and permission settings.",
        "contact-form" =>
            "A contact form block with name, email, subject, and message fields.",
        "profile-form" =>
            "A profile settings form with avatar upload, personal info, and social links.",
        _ => "A pre-built UI block ready to use in your application.",
    }
}

fn get_block_preview(block: &str) -> &'static str {
    match block {
        "login" | "login-01" => r##"
            <div class="w-full max-w-sm">
                <div class="rounded-xl border bg-card p-6 shadow-sm">
                    <div class="mb-6 text-center">
                        <h3 class="text-xl font-semibold">Welcome back</h3>
                        <p class="text-sm text-muted-foreground mt-1">Sign in to your account</p>
                    </div>
                    <div class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium">Email</label>
                            <input type="email" placeholder="you@example.com" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium">Password</label>
                            <input type="password" placeholder="••••••••" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                        </div>
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">
                            Sign in
                        </button>
                    </div>
                </div>
            </div>
        "##,
        "login-02" => r##"
            <div class="w-full max-w-sm">
                <div class="rounded-xl border bg-card p-6 shadow-sm">
                    <div class="mb-6 text-center">
                        <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
                            <svg class="w-6 h-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                        </div>
                        <h3 class="text-xl font-semibold">Sign in</h3>
                    </div>
                    <div class="space-y-4">
                        <button class="inline-flex items-center justify-center gap-2 w-full rounded-md text-sm font-medium h-10 px-4 py-2 border border-input bg-background hover:bg-accent">
                            <svg class="w-4 h-4" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>
                            Continue with Google
                        </button>
                        <button class="inline-flex items-center justify-center gap-2 w-full rounded-md text-sm font-medium h-10 px-4 py-2 border border-input bg-background hover:bg-accent">
                            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                            Continue with GitHub
                        </button>
                        <div class="relative my-4">
                            <div class="absolute inset-0 flex items-center"><span class="w-full border-t"></span></div>
                            <div class="relative flex justify-center text-xs uppercase"><span class="bg-background px-2 text-muted-foreground">Or continue with</span></div>
                        </div>
                        <input type="email" placeholder="Email" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm">
                        <input type="password" placeholder="Password" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm">
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Sign in</button>
                    </div>
                </div>
            </div>
        "##,
        "login-03" => r##"
            <div class="w-full max-w-4xl flex rounded-xl border bg-card shadow-sm overflow-hidden">
                <div class="flex-1 bg-primary/5 p-8 hidden md:flex flex-col justify-center">
                    <div class="mb-8">
                        <div class="w-10 h-10 rounded-lg bg-primary flex items-center justify-center mb-4">
                            <svg class="w-6 h-6 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                        </div>
                        <h2 class="text-2xl font-bold">Acme Inc</h2>
                        <p class="text-muted-foreground mt-2">Manage your business with our powerful platform.</p>
                    </div>
                    <blockquote class="border-l-2 border-primary pl-4 italic text-muted-foreground">"This platform has transformed how we work. Highly recommended!"</blockquote>
                </div>
                <div class="flex-1 p-8">
                    <div class="mb-6"><h3 class="text-xl font-semibold">Sign in to your account</h3></div>
                    <div class="space-y-4">
                        <div class="space-y-2"><label class="text-sm font-medium">Email</label><input type="email" placeholder="name@example.com" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                        <div class="space-y-2"><label class="text-sm font-medium">Password</label><input type="password" placeholder="••••••••" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Sign in</button>
                    </div>
                </div>
            </div>
        "##,
        "login-04" => r##"
            <div class="w-full max-w-sm">
                <div class="rounded-xl border bg-card p-6 shadow-sm">
                    <div class="mb-6">
                        <h3 class="text-xl font-semibold">Login</h3>
                        <p class="text-sm text-muted-foreground mt-1">Enter your credentials below</p>
                    </div>
                    <div class="space-y-4">
                        <div class="flex gap-2">
                            <button class="flex-1 inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium h-10 px-4 py-2 border border-input bg-background hover:bg-accent">
                                <svg class="w-4 h-4" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>
                            </button>
                            <button class="flex-1 inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium h-10 px-4 py-2 border border-input bg-background hover:bg-accent">
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M16.365 1.43c0 1.14-.493 2.27-1.177 3.08-.744.9-1.99 1.57-2.987 1.57-.12 0-.23-.02-.3-.03-.01-.06-.04-.22-.04-.39 0-1.15.572-2.27 1.206-2.98.804-.94 2.142-1.64 3.248-1.68.03.13.05.28.05.43zm4.565 15.71c-.03.07-.463 1.58-1.518 3.12-.945 1.34-1.94 2.71-3.43 2.71-1.517 0-1.9-.88-3.63-.88-1.698 0-2.302.91-3.67.91-1.377 0-2.332-1.26-3.428-2.8-1.287-1.82-2.323-4.63-2.323-7.28 0-4.28 2.797-6.55 5.552-6.55 1.448 0 2.675.95 3.6.95.865 0 2.222-1.01 3.902-1.01.613 0 2.886.06 4.374 2.19-.13.09-2.383 1.37-2.383 4.19 0 3.26 2.854 4.42 2.955 4.45z"/></svg>
                            </button>
                        </div>
                        <div class="relative"><div class="absolute inset-0 flex items-center"><span class="w-full border-t"></span></div><div class="relative flex justify-center text-xs uppercase"><span class="bg-background px-2 text-muted-foreground">Or</span></div></div>
                        <input type="email" placeholder="Email" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm">
                        <input type="password" placeholder="Password" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm">
                        <div class="flex items-center justify-between text-sm"><label class="flex items-center gap-2"><input type="checkbox" class="rounded border-input"> Remember me</label><a href="#" class="text-primary hover:underline">Forgot password?</a></div>
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Login</button>
                    </div>
                </div>
            </div>
        "##,
        "login-05" => r##"
            <div class="w-full max-w-sm">
                <div class="text-center mb-8">
                    <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center mx-auto mb-4">
                        <svg class="w-8 h-8 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
                    </div>
                    <h3 class="text-2xl font-bold">Welcome to App</h3>
                    <p class="text-muted-foreground mt-1">Enter your email to get started</p>
                </div>
                <div class="space-y-4">
                    <input type="email" placeholder="Email address" class="flex h-11 w-full rounded-lg border border-input bg-transparent px-4 py-2 text-sm">
                    <button class="inline-flex items-center justify-center w-full rounded-lg text-sm font-medium h-11 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Continue with Email</button>
                    <div class="relative"><div class="absolute inset-0 flex items-center"><span class="w-full border-t"></span></div><div class="relative flex justify-center text-xs uppercase"><span class="bg-background px-2 text-muted-foreground">Or</span></div></div>
                    <button class="inline-flex items-center justify-center gap-2 w-full rounded-lg text-sm font-medium h-11 px-4 py-2 border border-input bg-background hover:bg-accent">
                        <svg class="w-5 h-5" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>
                        Continue with Google
                    </button>
                </div>
            </div>
        "##,
        "signup" | "signup-01" => r##"
            <div class="w-full max-w-sm">
                <div class="rounded-xl border bg-card p-6 shadow-sm">
                    <div class="mb-6 text-center">
                        <h3 class="text-xl font-semibold">Create account</h3>
                        <p class="text-sm text-muted-foreground mt-1">Get started for free</p>
                    </div>
                    <div class="space-y-4">
                        <div class="space-y-2">
                            <label class="text-sm font-medium">Name</label>
                            <input type="text" placeholder="John Doe" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium">Email</label>
                            <input type="email" placeholder="you@example.com" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                        </div>
                        <div class="space-y-2">
                            <label class="text-sm font-medium">Password</label>
                            <input type="password" placeholder="••••••••" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                        </div>
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">
                            Create account
                        </button>
                    </div>
                </div>
            </div>
        "##,
        "otp-01" => r##"
            <div class="w-full max-w-sm">
                <div class="rounded-xl border bg-card p-6 shadow-sm">
                    <div class="mb-6 text-center">
                        <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
                            <svg class="w-6 h-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                        </div>
                        <h3 class="text-xl font-semibold">Verify your email</h3>
                        <p class="text-sm text-muted-foreground mt-1">Enter the 6-digit code sent to your email</p>
                    </div>
                    <div class="space-y-4">
                        <div class="flex justify-center gap-2">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                            <input type="text" maxlength="1" class="w-10 h-12 text-center text-xl font-semibold rounded-md border border-input bg-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                        </div>
                        <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Verify</button>
                        <p class="text-center text-sm text-muted-foreground">Didn't receive a code? <a href="#" class="text-primary hover:underline">Resend</a></p>
                    </div>
                </div>
            </div>
        "##,
        "dashboard-01" => r##"
            <div class="w-full max-w-2xl">
                <div class="grid grid-cols-3 gap-4 mb-6">
                    <div class="rounded-lg border bg-card p-4">
                        <p class="text-sm text-muted-foreground">Total Users</p>
                        <p class="text-2xl font-bold">2,543</p>
                        <p class="text-xs text-green-500">+12.5%</p>
                    </div>
                    <div class="rounded-lg border bg-card p-4">
                        <p class="text-sm text-muted-foreground">Revenue</p>
                        <p class="text-2xl font-bold">$45.2k</p>
                        <p class="text-xs text-green-500">+8.2%</p>
                    </div>
                    <div class="rounded-lg border bg-card p-4">
                        <p class="text-sm text-muted-foreground">Active Now</p>
                        <p class="text-2xl font-bold">573</p>
                        <p class="text-xs text-muted-foreground">+201 today</p>
                    </div>
                </div>
                <div class="rounded-lg border bg-card">
                    <div class="p-4 border-b">
                        <h4 class="font-semibold">Recent Activity</h4>
                    </div>
                    <div class="p-4 space-y-3">
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center"><svg class="w-4 h-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg></div>
                            <div class="flex-1">
                                <p class="text-sm">New user registered</p>
                                <p class="text-xs text-muted-foreground">2 minutes ago</p>
                            </div>
                        </div>
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center"><svg class="w-4 h-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg></div>
                            <div class="flex-1">
                                <p class="text-sm">Payment received - $250.00</p>
                                <p class="text-xs text-muted-foreground">15 minutes ago</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        "##,
        "sidebar-01" | "sidebar-02" | "sidebar-03" => r##"
            <div class="w-64 h-[400px] rounded-lg border bg-card overflow-hidden">
                <div class="p-4 border-b">
                    <div class="flex items-center gap-2">
                        <div class="w-8 h-8 rounded-lg bg-primary flex items-center justify-center"><svg class="w-4 h-4 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg></div>
                        <span class="font-semibold">Acme Inc</span>
                    </div>
                </div>
                <nav class="p-2 space-y-1">
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md bg-primary/10 text-primary"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg><span class="text-sm font-medium">Dashboard</span></a>
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/></svg><span class="text-sm">Users</span></a>
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/></svg><span class="text-sm">Billing</span></a>
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg><span class="text-sm">Settings</span></a>
                </nav>
            </div>
        "##,
        "sidebar-07" => r##"
            <div class="w-64 h-[400px] rounded-lg border bg-card overflow-hidden flex flex-col">
                <div class="p-4 border-b">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 rounded-full bg-muted"></div>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-medium truncate">John Doe</p>
                            <p class="text-xs text-muted-foreground truncate">john@example.com</p>
                        </div>
                    </div>
                </div>
                <nav class="flex-1 p-2 space-y-1">
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md bg-primary/10 text-primary"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg><span class="text-sm font-medium">Home</span></a>
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/></svg><span class="text-sm">Projects</span></a>
                    <a href="#" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg><span class="text-sm">Tasks</span></a>
                </nav>
                <div class="p-2 border-t">
                    <button class="flex items-center gap-3 px-3 py-2 w-full rounded-md hover:bg-muted text-muted-foreground"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg><span class="text-sm">Log out</span></button>
                </div>
            </div>
        "##,
        "sidebar-11" => r##"
            <div class="w-64 h-[400px] rounded-lg border bg-card overflow-hidden flex flex-col">
                <div class="p-4 border-b">
                    <div class="relative"><svg class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg><input type="search" placeholder="Search files..." class="w-full h-9 pl-8 pr-3 rounded-md border border-input bg-transparent text-sm"></div>
                </div>
                <div class="flex-1 p-2 overflow-auto">
                    <div x-data="{ open: true }" class="space-y-1">
                        <button @click="open = !open" class="flex items-center gap-2 px-2 py-1 w-full rounded hover:bg-muted text-sm"><svg class="w-4 h-4" :class="open ? 'rotate-90' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg><svg class="w-4 h-4 text-blue-500" viewBox="0 0 24 24" fill="currentColor"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/></svg><span>src</span></button>
                        <div x-show="open" class="pl-6 space-y-1">
                            <a href="#" class="flex items-center gap-2 px-2 py-1 rounded hover:bg-muted text-sm text-muted-foreground"><svg class="w-4 h-4 text-yellow-500" viewBox="0 0 24 24" fill="currentColor"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/></svg>main.rs</a>
                            <a href="#" class="flex items-center gap-2 px-2 py-1 rounded hover:bg-muted text-sm text-muted-foreground"><svg class="w-4 h-4 text-yellow-500" viewBox="0 0 24 24" fill="currentColor"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/></svg>lib.rs</a>
                        </div>
                    </div>
                </div>
            </div>
        "##,
        "sidebar-12" => r##"
            <div class="w-72 h-[400px] rounded-lg border bg-card overflow-hidden flex flex-col">
                <div class="p-4 border-b">
                    <div class="flex items-center justify-between mb-4">
                        <button class="p-1 hover:bg-muted rounded"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                        <span class="font-medium">January 2024</span>
                        <button class="p-1 hover:bg-muted rounded"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                    </div>
                    <div class="grid grid-cols-7 gap-1 text-center text-xs">
                        <span class="text-muted-foreground">Su</span><span class="text-muted-foreground">Mo</span><span class="text-muted-foreground">Tu</span><span class="text-muted-foreground">We</span><span class="text-muted-foreground">Th</span><span class="text-muted-foreground">Fr</span><span class="text-muted-foreground">Sa</span>
                        <span class="py-1 text-muted-foreground">31</span><span class="py-1">1</span><span class="py-1">2</span><span class="py-1">3</span><span class="py-1">4</span><span class="py-1">5</span><span class="py-1">6</span>
                        <span class="py-1">7</span><span class="py-1">8</span><span class="py-1">9</span><span class="py-1">10</span><span class="py-1">11</span><span class="py-1">12</span><span class="py-1">13</span>
                        <span class="py-1">14</span><span class="py-1 rounded bg-primary text-primary-foreground">15</span><span class="py-1">16</span><span class="py-1">17</span><span class="py-1">18</span><span class="py-1">19</span><span class="py-1">20</span>
                    </div>
                </div>
                <div class="flex-1 p-4 overflow-auto">
                    <h4 class="text-sm font-medium mb-3">Upcoming Events</h4>
                    <div class="space-y-2">
                        <div class="p-2 rounded border-l-2 border-blue-500 bg-blue-500/10"><p class="text-sm font-medium">Team Meeting</p><p class="text-xs text-muted-foreground">10:00 AM</p></div>
                        <div class="p-2 rounded border-l-2 border-green-500 bg-green-500/10"><p class="text-sm font-medium">Project Review</p><p class="text-xs text-muted-foreground">2:00 PM</p></div>
                    </div>
                </div>
            </div>
        "##,
        "sidebar-15" => r##"
            <div class="w-64 h-[400px] rounded-lg border bg-card overflow-hidden flex flex-col">
                <div class="p-4 border-b"><h3 class="font-semibold">Settings</h3></div>
                <nav class="flex-1 p-2 space-y-1">
                    <a href="#" class="flex items-center justify-between px-3 py-2 rounded-md bg-primary/10 text-primary"><span class="flex items-center gap-3"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg><span class="text-sm font-medium">Account</span></span></a>
                    <a href="#" class="flex items-center justify-between px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><span class="flex items-center gap-3"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="11" x="3" y="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg><span class="text-sm">Security</span></span></a>
                    <a href="#" class="flex items-center justify-between px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><span class="flex items-center gap-3"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/></svg><span class="text-sm">Notifications</span></span><span class="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground">3</span></a>
                    <a href="#" class="flex items-center justify-between px-3 py-2 rounded-md hover:bg-muted text-muted-foreground"><span class="flex items-center gap-3"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/></svg><span class="text-sm">Billing</span></span></a>
                </nav>
            </div>
        "##,
        "confirm-dialog" => r##"
            <div class="w-full max-w-md rounded-lg border bg-card p-6 shadow-lg">
                <div class="mb-4">
                    <h3 class="text-lg font-semibold">Confirm Action</h3>
                    <p class="text-sm text-muted-foreground mt-1">Are you sure you want to proceed with this action? This cannot be undone.</p>
                </div>
                <div class="flex justify-end gap-2">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background hover:bg-accent">Cancel</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Confirm</button>
                </div>
            </div>
        "##,
        "delete-dialog" => r##"
            <div class="w-full max-w-md rounded-lg border bg-card p-6 shadow-lg">
                <div class="flex items-start gap-4 mb-4">
                    <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center shrink-0">
                        <svg class="w-5 h-5 text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                    </div>
                    <div>
                        <h3 class="text-lg font-semibold">Delete Item</h3>
                        <p class="text-sm text-muted-foreground mt-1">This action cannot be undone. This will permanently delete the item and remove all associated data.</p>
                    </div>
                </div>
                <div class="flex justify-end gap-2">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background hover:bg-accent">Cancel</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-destructive text-destructive-foreground shadow hover:bg-destructive/90">Delete</button>
                </div>
            </div>
        "##,
        "share-dialog" => r##"
            <div class="w-full max-w-md rounded-lg border bg-card p-6 shadow-lg">
                <div class="mb-4">
                    <h3 class="text-lg font-semibold">Share</h3>
                    <p class="text-sm text-muted-foreground mt-1">Share this item with others</p>
                </div>
                <div class="space-y-4">
                    <div class="flex gap-2">
                        <input type="text" value="https://example.com/share/abc123" readonly class="flex-1 h-9 rounded-md border border-input bg-muted/50 px-3 py-1 text-sm">
                        <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-3 border border-input bg-background hover:bg-accent"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="14" height="14" x="8" y="8" rx="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg></button>
                    </div>
                    <div class="flex gap-2">
                        <input type="email" placeholder="Email address" class="flex-1 h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm">
                        <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Send</button>
                    </div>
                </div>
            </div>
        "##,
        "contact-form" => r##"
            <div class="w-full max-w-md rounded-lg border bg-card p-6 shadow-sm">
                <div class="mb-6">
                    <h3 class="text-xl font-semibold">Contact Us</h3>
                    <p class="text-sm text-muted-foreground mt-1">Fill out the form and we'll get back to you</p>
                </div>
                <div class="space-y-4">
                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2"><label class="text-sm font-medium">First name</label><input type="text" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                        <div class="space-y-2"><label class="text-sm font-medium">Last name</label><input type="text" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                    </div>
                    <div class="space-y-2"><label class="text-sm font-medium">Email</label><input type="email" placeholder="you@example.com" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                    <div class="space-y-2"><label class="text-sm font-medium">Subject</label><input type="text" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                    <div class="space-y-2"><label class="text-sm font-medium">Message</label><textarea rows="4" class="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm"></textarea></div>
                    <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Send Message</button>
                </div>
            </div>
        "##,
        "profile-form" => r##"
            <div class="w-full max-w-md rounded-lg border bg-card p-6 shadow-sm">
                <div class="mb-6">
                    <h3 class="text-xl font-semibold">Profile</h3>
                    <p class="text-sm text-muted-foreground mt-1">Manage your profile information</p>
                </div>
                <div class="space-y-6">
                    <div class="flex items-center gap-4">
                        <div class="w-16 h-16 rounded-full bg-muted flex items-center justify-center"><svg class="w-8 h-8 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg></div>
                        <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input bg-background hover:bg-accent">Change Photo</button>
                    </div>
                    <div class="space-y-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="space-y-2"><label class="text-sm font-medium">First name</label><input type="text" value="John" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                            <div class="space-y-2"><label class="text-sm font-medium">Last name</label><input type="text" value="Doe" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                        </div>
                        <div class="space-y-2"><label class="text-sm font-medium">Email</label><input type="email" value="john@example.com" class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"></div>
                        <div class="space-y-2"><label class="text-sm font-medium">Bio</label><textarea rows="3" placeholder="Tell us about yourself..." class="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm"></textarea></div>
                    </div>
                    <button class="inline-flex items-center justify-center w-full rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Save Changes</button>
                </div>
            </div>
        "##,
        "calendar-01" => r##"
            <div class="w-full max-w-3xl rounded-lg border bg-card shadow-sm overflow-hidden">
                <div class="p-4 border-b flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <button class="p-2 hover:bg-muted rounded-md"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                        <button class="p-2 hover:bg-muted rounded-md"><svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                        <h2 class="text-lg font-semibold ml-2">January 2024</h2>
                    </div>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 bg-primary text-primary-foreground shadow hover:bg-primary/90">Today</button>
                </div>
                <div class="grid grid-cols-7 border-b">
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Sun</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Mon</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Tue</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Wed</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Thu</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-r">Fri</div>
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground">Sat</div>
                </div>
                <div class="grid grid-cols-7">
                    <div class="h-24 p-2 border-r border-b text-muted-foreground">31</div>
                    <div class="h-24 p-2 border-r border-b">1</div>
                    <div class="h-24 p-2 border-r border-b">2</div>
                    <div class="h-24 p-2 border-r border-b">3</div>
                    <div class="h-24 p-2 border-r border-b">4</div>
                    <div class="h-24 p-2 border-r border-b">5</div>
                    <div class="h-24 p-2 border-b">6</div>
                    <div class="h-24 p-2 border-r border-b">7</div>
                    <div class="h-24 p-2 border-r border-b">8</div>
                    <div class="h-24 p-2 border-r border-b">9</div>
                    <div class="h-24 p-2 border-r border-b relative">10<div class="absolute bottom-1 left-1 right-1 text-xs p-1 rounded bg-blue-500/20 text-blue-600 truncate">Team Meeting</div></div>
                    <div class="h-24 p-2 border-r border-b">11</div>
                    <div class="h-24 p-2 border-r border-b">12</div>
                    <div class="h-24 p-2 border-b">13</div>
                </div>
            </div>
        "##,
        _ => r##"
            <div class="text-center p-8">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted-foreground mx-auto mb-4">
                    <rect width="7" height="7" x="3" y="3" rx="1"/>
                    <rect width="7" height="7" x="14" y="3" rx="1"/>
                    <rect width="7" height="7" x="14" y="14" rx="1"/>
                    <rect width="7" height="7" x="3" y="14" rx="1"/>
                </svg>
                <h4 class="text-lg font-medium mb-2">Block Preview</h4>
                <p class="text-sm text-muted-foreground">Interactive preview for this block.</p>
            </div>
        "##,
    }
}

fn get_block_usage(block: &str) -> &'static str {
    match block {
        "login" | "login-01" => r#"@import /blocks/auth/login-01.wtz as login

<@login::apply
    action="/api/login"
    method="post"
    forgot_password_url=Some("/forgot-password")
    signup_url=Some("/signup")
/>"#,
        "signup" | "signup-01" => r#"@import /blocks/auth/signup-01.wtz as signup

<@signup::apply
    action="/api/signup"
    method="post"
    login_url=Some("/login")
    terms_url=Some("/terms")
/>"#,
        "dashboard-01" => r#"@import /blocks/dashboard/dashboard-01.wtz as dashboard

@let stats = vec![
    dashboard::Stat { label: "Users", value: "2,543", change: Some("+12%") },
    dashboard::Stat { label: "Revenue", value: "$45k", change: Some("+8%") },
]

<@dashboard::apply stats=@stats />"#,
        "sidebar-01" | "sidebar-02" | "sidebar-03" => r#"@import /blocks/sidebar/sidebar-01.wtz as sidebar

@let items = vec![
    sidebar::NavItem { label: "Dashboard", href: "/", icon: Some("..."), active: true },
    sidebar::NavItem { label: "Settings", href: "/settings", icon: Some("..."), active: false },
]

<@sidebar::apply nav_items=@items />"#,
        "calendar-01" => r#"@import /blocks/calendar/calendar-01.wtz as calendar

@let events = vec![
    calendar::Event { date: "2024-03-15", title: "Meeting", color: None },
]

<@calendar::apply events=@events />"#,
        _ => r#"@import /blocks/CATEGORY/BLOCK_NAME.wtz as block

<@block::apply
    // ... block parameters
/>"#,
    }
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
            <div x-data="{
                open: false,
                selected: 'Select a fruit',
                pos: { x: 0, y: 0, width: 0 },
                flipTop: false,
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const dropdownHeight = 150;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < dropdownHeight && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                    this.pos.width = rect.width;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="w-[180px]">
                <button x-ref="trigger" @click="open = !open; $nextTick(() => updatePos())" class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring">
                    <span x-text="selected"></span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-50"><path d="m6 9 6 6 6-6"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-[9999] rounded-md border border-border bg-popover text-popover-foreground p-1 shadow-md" :style="`left: ${pos.x}px; width: ${pos.width}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 4) + 'px' : 'top: ' + (pos.y + 4) + 'px'}`">
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
            <div x-data="{
                open: false,
                pos: { x: 0, y: 0 },
                flipTop: false,
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const dropdownHeight = 200;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < dropdownHeight && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="inline-block">
                <button x-ref="trigger" @click="open = !open; $nextTick(() => updatePos())" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">
                    Open Menu
                    <svg class="ml-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed w-56 rounded-md border border-border bg-popover text-popover-foreground p-1 shadow-md z-[9999]" :style="`left: ${pos.x}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 4) + 'px' : 'top: ' + (pos.y + 4) + 'px'}`">
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
            <div x-data="{
                open: false,
                pos: { x: 0, y: 0 },
                flipTop: false,
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const popoverHeight = 200;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < popoverHeight && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="inline-block">
                <button x-ref="trigger" @click="open = !open; $nextTick(() => updatePos())" class="inline-flex items-center justify-center rounded-md text-sm font-medium h-9 px-4 py-2 border border-input hover:bg-accent">Open Popover</button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed w-80 rounded-md border border-border bg-popover text-popover-foreground p-4 shadow-md z-[9999]" :style="`left: ${pos.x}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 8) + 'px' : 'top: ' + (pos.y + 8) + 'px'}`">
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
            <div x-data="{
                open: false,
                search: '',
                selected: '',
                pos: { x: 0, y: 0, width: 0 },
                flipTop: false,
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const dropdownHeight = 150;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < dropdownHeight && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                    this.pos.width = rect.width;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="w-[200px]">
                <button x-ref="trigger" @click="open = !open; $nextTick(() => updatePos())" class="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm">
                    <span x-text="selected || 'Select framework...'"></span>
                    <svg class="h-4 w-4 opacity-50" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-[9999] rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="`left: ${pos.x}px; width: ${pos.width}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 4) + 'px' : 'top: ' + (pos.y + 4) + 'px'}`">
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
            <div x-data="{
                open: false,
                value: '',
                displayValue: '',
                currentMonth: new Date().getMonth(),
                currentYear: new Date().getFullYear(),
                currentView: 'days',
                yearInput: '',
                minYear: new Date().getFullYear() - 100,
                maxYear: new Date().getFullYear() + 20,
                days: [],
                weekdays: ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
                months: ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'],
                monthsShort: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'],
                pos: { x: 0, y: 0 },
                flipTop: false,
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const calendarHeight = 380;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < calendarHeight && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                    this.yearInput = this.currentYear;
                    this.buildCalendar();
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                },
                formatISODate(date) {
                    const year = date.getFullYear();
                    const month = String(date.getMonth() + 1).padStart(2, '0');
                    const day = String(date.getDate()).padStart(2, '0');
                    return year + '-' + month + '-' + day;
                },
                formatDisplayValue(date) {
                    this.displayValue = date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
                },
                setView(view) {
                    this.currentView = view;
                    if (view === 'years') {
                        this.yearInput = this.currentYear;
                        this.updateYearRange();
                    }
                },
                prevPeriod() {
                    if (this.currentView === 'days') { this.prevMonth(); }
                    else if (this.currentView === 'months') { this.currentYear--; this.yearInput = this.currentYear; }
                    else if (this.currentView === 'years') { this.minYear -= 20; this.maxYear -= 20; }
                },
                nextPeriod() {
                    if (this.currentView === 'days') { this.nextMonth(); }
                    else if (this.currentView === 'months') { this.currentYear++; this.yearInput = this.currentYear; }
                    else if (this.currentView === 'years') { this.minYear += 20; this.maxYear += 20; }
                },
                prevMonth() {
                    if (this.currentMonth === 0) { this.currentMonth = 11; this.currentYear--; }
                    else { this.currentMonth--; }
                    this.buildCalendar();
                },
                nextMonth() {
                    if (this.currentMonth === 11) { this.currentMonth = 0; this.currentYear++; }
                    else { this.currentMonth++; }
                    this.buildCalendar();
                },
                buildCalendar() {
                    this.days = [];
                    const firstDay = new Date(this.currentYear, this.currentMonth, 1);
                    const startDate = new Date(firstDay);
                    startDate.setDate(startDate.getDate() - firstDay.getDay());
                    for (let i = 0; i < 42; i++) {
                        const date = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate() + i);
                        const isoDate = this.formatISODate(date);
                        const isCurrentMonth = date.getMonth() === this.currentMonth;
                        const isSelected = this.value === isoDate;
                        const isToday = this.formatISODate(new Date()) === isoDate;
                        this.days.push({ day: date.getDate(), date: isoDate, selected: isSelected, today: isToday, currentMonth: isCurrentMonth });
                    }
                },
                selectDate(day) {
                    this.value = day.date;
                    const date = new Date(day.date + 'T00:00:00');
                    this.currentMonth = date.getMonth();
                    this.currentYear = date.getFullYear();
                    this.formatDisplayValue(date);
                    this.buildCalendar();
                    this.open = false;
                },
                selectMonth(monthIndex) {
                    this.currentMonth = monthIndex;
                    this.buildCalendar();
                    this.setView('days');
                },
                selectYear(year) {
                    this.currentYear = year;
                    this.yearInput = year;
                    this.buildCalendar();
                    this.setView('months');
                },
                selectYearFromInput() {
                    const year = parseInt(this.yearInput);
                    if (this.isValidYearInput()) { this.selectYear(year); }
                },
                clearYearInput() { this.yearInput = ''; },
                isValidYearInput() {
                    const year = parseInt(this.yearInput);
                    return year && year >= 1000 && year <= 9999 && !isNaN(year);
                },
                validateYearKeypress(event) {
                    const char = String.fromCharCode(event.which);
                    if (!/[0-9]/.test(char)) { event.preventDefault(); }
                },
                handleYearInput(event) {
                    const value = event.target.value.replace(/[^0-9]/g, '');
                    this.yearInput = value;
                    this.updateYearRange();
                },
                updateYearRange() {
                    const inputYear = parseInt(this.yearInput);
                    if (inputYear && (inputYear < this.minYear || inputYear > this.maxYear)) {
                        this.minYear = Math.max(1000, inputYear - 10);
                        this.maxYear = Math.min(9999, inputYear + 10);
                    }
                },
                getYearOptions() {
                    const years = [];
                    for (let year = this.minYear; year <= this.maxYear; year++) { years.push(year); }
                    return years;
                },
                selectToday() {
                    const today = new Date();
                    const isoDate = this.formatISODate(today);
                    this.value = isoDate;
                    this.currentMonth = today.getMonth();
                    this.currentYear = today.getFullYear();
                    this.formatDisplayValue(today);
                    this.buildCalendar();
                    this.open = false;
                }
            }" x-init="init()" @destroy="destroy()" class="inline-block">
                <button x-ref="trigger" @click="open = !open; currentView = 'days'; $nextTick(() => updatePos())" class="flex h-9 w-[200px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                    <span :class="displayValue ? '' : 'text-muted-foreground'" x-text="displayValue || 'Pick a date'"></span>
                    <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-[9999] w-80 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="`left: ${pos.x}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 8) + 'px' : 'top: ' + (pos.y + 8) + 'px'}`">
                        <!-- Header with navigation -->
                        <div class="flex items-center justify-between p-3 border-b">
                            <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent opacity-50 hover:opacity-100" @click="prevPeriod()"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                            <div class="flex items-center gap-1">
                                <button type="button" class="px-2 py-1 text-sm font-medium rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'years' }" @click="setView('years')" x-text="currentYear"></button>
                                <button type="button" class="px-2 py-1 text-sm font-medium rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'months' }" @click="setView('months')" x-text="monthsShort[currentMonth]"></button>
                                <button type="button" class="p-1 rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'days' }" @click="setView('days')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg></button>
                            </div>
                            <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent opacity-50 hover:opacity-100" @click="nextPeriod()"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                        </div>
                        <div class="p-3">
                            <!-- Days View -->
                            <div x-show="currentView === 'days'">
                                <div class="grid grid-cols-7 gap-1 mb-1">
                                    <template x-for="weekday in weekdays" :key="weekday">
                                        <div class="text-center text-xs text-muted-foreground font-medium h-8 flex items-center justify-center" x-text="weekday"></div>
                                    </template>
                                </div>
                                <div class="grid grid-cols-7 gap-1">
                                    <template x-for="(day, index) in days" :key="index">
                                        <button type="button" class="h-8 w-8 text-sm rounded-md flex items-center justify-center transition-colors" :class="{ 'bg-primary text-primary-foreground': day.selected, 'bg-accent': day.today && !day.selected, 'text-muted-foreground opacity-50': !day.currentMonth, 'hover:bg-accent': !day.selected && day.currentMonth }" @click="selectDate(day)" x-text="day.day"></button>
                                    </template>
                                </div>
                            </div>
                            <!-- Months View -->
                            <div x-show="currentView === 'months'">
                                <div class="grid grid-cols-3 gap-2">
                                    <template x-for="(month, index) in monthsShort" :key="index">
                                        <button type="button" class="px-3 py-2 text-sm font-medium rounded-md transition-colors" :class="{ 'bg-primary text-primary-foreground': index === currentMonth, 'hover:bg-accent': index !== currentMonth }" @click="selectMonth(index)" x-text="month"></button>
                                    </template>
                                </div>
                            </div>
                            <!-- Years View -->
                            <div x-show="currentView === 'years'">
                                <div class="space-y-3">
                                    <div class="relative">
                                        <input type="text" x-model="yearInput" @keydown.enter="selectYearFromInput()" @input="handleYearInput($event)" @keypress="validateYearKeypress($event)" placeholder="Enter year" maxlength="4" class="w-full px-3 py-2 pr-16 border border-input rounded-md text-center font-medium bg-transparent text-sm focus:outline-none focus:ring-1 focus:ring-ring" />
                                        <div x-show="yearInput && yearInput.toString().length > 0" class="absolute right-1 top-1/2 -translate-y-1/2 flex gap-1">
                                            <button type="button" @click="clearYearInput()" class="p-1 hover:bg-accent rounded transition-colors" title="Clear"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-muted-foreground"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
                                            <button type="button" @click="selectYearFromInput()" :disabled="!isValidYearInput()" class="p-1 hover:bg-accent rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed" :class="{ 'text-primary': isValidYearInput(), 'text-muted-foreground': !isValidYearInput() }" title="Select year"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6 9 17l-5-5"/></svg></button>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-4 gap-2 max-h-48 overflow-y-auto">
                                        <template x-for="year in getYearOptions()" :key="year">
                                            <button type="button" class="px-2 py-1.5 text-sm font-medium rounded-md transition-colors" :class="{ 'bg-primary text-primary-foreground': year === currentYear, 'hover:bg-accent': year !== currentYear }" @click="selectYear(year)" x-text="year"></button>
                                        </template>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <!-- Today Button -->
                        <div class="p-3 pt-0">
                            <button type="button" class="w-full h-8 px-3 text-sm font-medium rounded-md bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-colors" @click="selectToday()">Today</button>
                        </div>
                    </div>
                </template>
            </div>
        "#,

        "datetime-picker" => r#"
            <div x-data="{
                dateOpen: false,
                timeOpen: false,
                dateValue: '',
                displayDate: '',
                currentMonth: new Date().getMonth(),
                currentYear: new Date().getFullYear(),
                currentView: 'days',
                yearInput: '',
                minYear: new Date().getFullYear() - 100,
                maxYear: new Date().getFullYear() + 20,
                days: [],
                weekdays: ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
                months: ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'],
                monthsShort: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'],
                hour: 9,
                minute: 0,
                second: 0,
                period: 'AM',
                use12h: true,
                showSeconds: false,
                datePos: { x: 0, y: 0 },
                timePos: { x: 0, y: 0 },
                dateFlipTop: false,
                timeFlipTop: false,
                get maxHour() { return this.use12h ? 12 : 23; },
                get minHour() { return this.use12h ? 1 : 0; },
                get displayHour() { return this.hour.toString().padStart(2, '0'); },
                get displayMinute() { return this.minute.toString().padStart(2, '0'); },
                get displaySecond() { return this.second.toString().padStart(2, '0'); },
                get displayTime() {
                    let t = this.displayHour + ':' + this.displayMinute;
                    if (this.showSeconds) t += ':' + this.displaySecond;
                    if (this.use12h) t += ' ' + this.period;
                    return t;
                },
                toggle12h() {
                    const was12h = this.use12h;
                    this.use12h = !this.use12h;
                    if (this.use12h && !was12h) {
                        this.period = this.hour >= 12 ? 'PM' : 'AM';
                        this.hour = this.hour % 12;
                        if (this.hour === 0) this.hour = 12;
                    } else if (!this.use12h && was12h) {
                        let h = this.hour % 12;
                        if (this.period === 'PM') h += 12;
                        if (h === 24) h = 0;
                        this.hour = h;
                    }
                },
                incHour() { this.hour = this.hour >= this.maxHour ? this.minHour : this.hour + 1; },
                decHour() { this.hour = this.hour <= this.minHour ? this.maxHour : this.hour - 1; },
                incMinute() { this.minute = this.minute >= 59 ? 0 : this.minute + 1; },
                decMinute() { this.minute = this.minute <= 0 ? 59 : this.minute - 1; },
                incSecond() { this.second = this.second >= 59 ? 0 : this.second + 1; },
                decSecond() { this.second = this.second <= 0 ? 59 : this.second - 1; },
                togglePeriod() { this.period = this.period === 'AM' ? 'PM' : 'AM'; },
                setHour(val) { const n = parseInt(val, 10); if (!isNaN(n) && n >= this.minHour && n <= this.maxHour) this.hour = n; },
                setMinute(val) { const n = parseInt(val, 10); if (!isNaN(n) && n >= 0 && n <= 59) this.minute = n; },
                setSecond(val) { const n = parseInt(val, 10); if (!isNaN(n) && n >= 0 && n <= 59) this.second = n; },
                formatISODate(date) {
                    const year = date.getFullYear();
                    const month = String(date.getMonth() + 1).padStart(2, '0');
                    const day = String(date.getDate()).padStart(2, '0');
                    return year + '-' + month + '-' + day;
                },
                formatDisplayDate(date) {
                    this.displayDate = date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
                },
                setView(view) {
                    this.currentView = view;
                    if (view === 'years') {
                        this.yearInput = this.currentYear;
                        this.updateYearRange();
                    }
                },
                prevPeriod() {
                    if (this.currentView === 'days') { this.prevMonth(); }
                    else if (this.currentView === 'months') { this.currentYear--; this.yearInput = this.currentYear; }
                    else if (this.currentView === 'years') { this.minYear -= 20; this.maxYear -= 20; }
                },
                nextPeriod() {
                    if (this.currentView === 'days') { this.nextMonth(); }
                    else if (this.currentView === 'months') { this.currentYear++; this.yearInput = this.currentYear; }
                    else if (this.currentView === 'years') { this.minYear += 20; this.maxYear += 20; }
                },
                prevMonth() {
                    if (this.currentMonth === 0) { this.currentMonth = 11; this.currentYear--; }
                    else { this.currentMonth--; }
                    this.buildCalendar();
                },
                nextMonth() {
                    if (this.currentMonth === 11) { this.currentMonth = 0; this.currentYear++; }
                    else { this.currentMonth++; }
                    this.buildCalendar();
                },
                buildCalendar() {
                    this.days = [];
                    const firstDay = new Date(this.currentYear, this.currentMonth, 1);
                    const startDate = new Date(firstDay);
                    startDate.setDate(startDate.getDate() - firstDay.getDay());
                    for (let i = 0; i < 42; i++) {
                        const date = new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate() + i);
                        const isoDate = this.formatISODate(date);
                        const isCurrentMonth = date.getMonth() === this.currentMonth;
                        const isSelected = this.dateValue === isoDate;
                        const isToday = this.formatISODate(new Date()) === isoDate;
                        this.days.push({ day: date.getDate(), date: isoDate, selected: isSelected, today: isToday, currentMonth: isCurrentMonth });
                    }
                },
                selectDate(day) {
                    this.dateValue = day.date;
                    const date = new Date(day.date + 'T00:00:00');
                    this.currentMonth = date.getMonth();
                    this.currentYear = date.getFullYear();
                    this.formatDisplayDate(date);
                    this.buildCalendar();
                    this.dateOpen = false;
                },
                selectMonth(monthIndex) {
                    this.currentMonth = monthIndex;
                    this.buildCalendar();
                    this.setView('days');
                },
                selectYear(year) {
                    this.currentYear = year;
                    this.yearInput = year;
                    this.buildCalendar();
                    this.setView('months');
                },
                selectYearFromInput() {
                    const year = parseInt(this.yearInput);
                    if (this.isValidYearInput()) { this.selectYear(year); }
                },
                clearYearInput() { this.yearInput = ''; },
                isValidYearInput() {
                    const year = parseInt(this.yearInput);
                    return year && year >= 1000 && year <= 9999 && !isNaN(year);
                },
                validateYearKeypress(event) {
                    const char = String.fromCharCode(event.which);
                    if (!/[0-9]/.test(char)) { event.preventDefault(); }
                },
                handleYearInput(event) {
                    const value = event.target.value.replace(/[^0-9]/g, '');
                    this.yearInput = value;
                    this.updateYearRange();
                },
                updateYearRange() {
                    const inputYear = parseInt(this.yearInput);
                    if (inputYear && (inputYear < this.minYear || inputYear > this.maxYear)) {
                        this.minYear = Math.max(1000, inputYear - 10);
                        this.maxYear = Math.min(9999, inputYear + 10);
                    }
                },
                getYearOptions() {
                    const years = [];
                    for (let year = this.minYear; year <= this.maxYear; year++) { years.push(year); }
                    return years;
                },
                selectToday() {
                    const today = new Date();
                    const isoDate = this.formatISODate(today);
                    this.dateValue = isoDate;
                    this.currentMonth = today.getMonth();
                    this.currentYear = today.getFullYear();
                    this.formatDisplayDate(today);
                    this.buildCalendar();
                    this.dateOpen = false;
                },
                updateDatePos() {
                    const trigger = this.$refs.dateBtn;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const height = 400;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.dateFlipTop = spaceBelow < height && spaceAbove > spaceBelow;
                    this.datePos.x = rect.left;
                    this.datePos.y = this.dateFlipTop ? rect.top : rect.bottom;
                },
                updateTimePos() {
                    const trigger = this.$refs.timeBtn;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const height = 200;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.timeFlipTop = spaceBelow < height && spaceAbove > spaceBelow;
                    this.timePos.x = rect.left;
                    this.timePos.y = this.timeFlipTop ? rect.top : rect.bottom;
                },
                onScroll() {
                    if (this.dateOpen) this.updateDatePos();
                    if (this.timeOpen) this.updateTimePos();
                },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                    this.yearInput = this.currentYear;
                    this.buildCalendar();
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="flex gap-2">
                <div>
                    <button x-ref="dateBtn" @click="dateOpen = !dateOpen; timeOpen = false; currentView = 'days'; $nextTick(() => updateDatePos())" class="flex h-9 w-[160px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                        <span :class="displayDate ? '' : 'text-muted-foreground'" x-text="displayDate || 'Date'"></span>
                        <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="dateOpen" @click.away="dateOpen = false" x-cloak class="fixed z-[9999] w-80 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="`left: ${datePos.x}px; ${dateFlipTop ? 'bottom: ' + (window.innerHeight - datePos.y + 8) + 'px' : 'top: ' + (datePos.y + 8) + 'px'}`">
                            <!-- Header with navigation -->
                            <div class="flex items-center justify-between p-3 border-b">
                                <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent opacity-50 hover:opacity-100" @click="prevPeriod()"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg></button>
                                <div class="flex items-center gap-1">
                                    <button type="button" class="px-2 py-1 text-sm font-medium rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'years' }" @click="setView('years')" x-text="currentYear"></button>
                                    <button type="button" class="px-2 py-1 text-sm font-medium rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'months' }" @click="setView('months')" x-text="monthsShort[currentMonth]"></button>
                                    <button type="button" class="p-1 rounded hover:bg-accent transition-colors" :class="{ 'bg-accent': currentView === 'days' }" @click="setView('days')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="4" rx="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg></button>
                                </div>
                                <button type="button" class="h-7 w-7 flex items-center justify-center rounded hover:bg-accent opacity-50 hover:opacity-100" @click="nextPeriod()"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg></button>
                            </div>
                            <div class="p-3">
                                <!-- Days View -->
                                <div x-show="currentView === 'days'">
                                    <div class="grid grid-cols-7 gap-1 mb-1">
                                        <template x-for="weekday in weekdays" :key="weekday">
                                            <div class="text-center text-xs text-muted-foreground font-medium h-8 flex items-center justify-center" x-text="weekday"></div>
                                        </template>
                                    </div>
                                    <div class="grid grid-cols-7 gap-1">
                                        <template x-for="(day, index) in days" :key="index">
                                            <button type="button" class="h-8 w-8 text-sm rounded-md flex items-center justify-center transition-colors" :class="{ 'bg-primary text-primary-foreground': day.selected, 'bg-accent': day.today && !day.selected, 'text-muted-foreground opacity-50': !day.currentMonth, 'hover:bg-accent': !day.selected && day.currentMonth }" @click="selectDate(day)" x-text="day.day"></button>
                                        </template>
                                    </div>
                                </div>
                                <!-- Months View -->
                                <div x-show="currentView === 'months'">
                                    <div class="grid grid-cols-3 gap-2">
                                        <template x-for="(month, index) in monthsShort" :key="index">
                                            <button type="button" class="px-3 py-2 text-sm font-medium rounded-md transition-colors" :class="{ 'bg-primary text-primary-foreground': index === currentMonth, 'hover:bg-accent': index !== currentMonth }" @click="selectMonth(index)" x-text="month"></button>
                                        </template>
                                    </div>
                                </div>
                                <!-- Years View -->
                                <div x-show="currentView === 'years'">
                                    <div class="space-y-3">
                                        <div class="relative">
                                            <input type="text" x-model="yearInput" @keydown.enter="selectYearFromInput()" @input="handleYearInput($event)" @keypress="validateYearKeypress($event)" placeholder="Enter year" maxlength="4" class="w-full px-3 py-2 pr-16 border border-input rounded-md text-center font-medium bg-transparent text-sm focus:outline-none focus:ring-1 focus:ring-ring" />
                                            <div x-show="yearInput && yearInput.toString().length > 0" class="absolute right-1 top-1/2 -translate-y-1/2 flex gap-1">
                                                <button type="button" @click="clearYearInput()" class="p-1 hover:bg-accent rounded transition-colors" title="Clear"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-muted-foreground"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg></button>
                                                <button type="button" @click="selectYearFromInput()" :disabled="!isValidYearInput()" class="p-1 hover:bg-accent rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed" :class="{ 'text-primary': isValidYearInput(), 'text-muted-foreground': !isValidYearInput() }" title="Select year"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6 9 17l-5-5"/></svg></button>
                                            </div>
                                        </div>
                                        <div class="grid grid-cols-4 gap-2 max-h-48 overflow-y-auto">
                                            <template x-for="year in getYearOptions()" :key="year">
                                                <button type="button" class="px-2 py-1.5 text-sm font-medium rounded-md transition-colors" :class="{ 'bg-primary text-primary-foreground': year === currentYear, 'hover:bg-accent': year !== currentYear }" @click="selectYear(year)" x-text="year"></button>
                                            </template>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <!-- Today Button -->
                            <div class="p-3 pt-0">
                                <button type="button" class="w-full h-8 px-3 text-sm font-medium rounded-md bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-colors" @click="selectToday()">Today</button>
                            </div>
                        </div>
                    </template>
                </div>
                <div>
                    <button x-ref="timeBtn" @click="timeOpen = !timeOpen; dateOpen = false; $nextTick(() => updateTimePos())" class="flex h-9 w-[140px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                        <span x-text="displayTime"></span>
                        <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="timeOpen" @click.away="timeOpen = false" x-cloak class="fixed z-[9999] p-3 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="`left: ${timePos.x}px; ${timeFlipTop ? 'bottom: ' + (window.innerHeight - timePos.y + 8) + 'px' : 'top: ' + (timePos.y + 8) + 'px'}`">
                            <div class="flex items-center justify-between mb-3 gap-2">
                                <button type="button" @click="toggle12h()" class="text-xs px-2 py-1 rounded border border-input hover:bg-accent" x-text="use12h ? '12h' : '24h'"></button>
                                <button type="button" @click="showSeconds = !showSeconds" class="text-xs px-2 py-1 rounded border border-input hover:bg-accent" x-text="showSeconds ? 'HH:MM:SS' : 'HH:MM'"></button>
                            </div>
                            <div class="flex items-center gap-1">
                                <div class="flex flex-col items-center">
                                    <button type="button" @click="incHour()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                    <input type="text" :value="displayHour" @blur="setHour($event.target.value); $event.target.value = displayHour" @keydown.up.prevent="incHour()" @keydown.down.prevent="decHour()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displayHour" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                    <button type="button" @click="decHour()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                </div>
                                <span class="text-lg font-bold">:</span>
                                <div class="flex flex-col items-center">
                                    <button type="button" @click="incMinute()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                    <input type="text" :value="displayMinute" @blur="setMinute($event.target.value); $event.target.value = displayMinute" @keydown.up.prevent="incMinute()" @keydown.down.prevent="decMinute()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displayMinute" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                    <button type="button" @click="decMinute()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                </div>
                                <template x-if="showSeconds">
                                    <span class="text-lg font-bold">:</span>
                                </template>
                                <template x-if="showSeconds">
                                    <div class="flex flex-col items-center">
                                        <button type="button" @click="incSecond()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                        <input type="text" :value="displaySecond" @blur="setSecond($event.target.value); $event.target.value = displaySecond" @keydown.up.prevent="incSecond()" @keydown.down.prevent="decSecond()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displaySecond" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                        <button type="button" @click="decSecond()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                    </div>
                                </template>
                                <template x-if="use12h">
                                    <div class="flex flex-col items-center ml-2">
                                        <button type="button" @click="togglePeriod()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                        <span class="text-lg font-mono w-10 text-center" x-text="period"></span>
                                        <button type="button" @click="togglePeriod()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                    </div>
                                </template>
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
            <nav x-data="{ activeNav: null, closeTimer: null }" class="relative flex items-center gap-1">
                <div class="relative">
                    <button @mouseenter="clearTimeout(closeTimer); activeNav = 'getting-started'" @mouseleave="closeTimer = setTimeout(() => activeNav = null, 150)" class="group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none">
                        Getting Started
                        <svg class="relative top-[1px] ml-1 h-3 w-3 transition duration-200" :class="activeNav === 'getting-started' ? 'rotate-180' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="activeNav === 'getting-started'" @mouseenter="clearTimeout(closeTimer); activeNav = 'getting-started'" @mouseleave="closeTimer = setTimeout(() => activeNav = null, 150)" x-cloak class="fixed left-1/2 z-50 w-[400px] -translate-x-1/2 rounded-md border bg-popover p-4 text-popover-foreground shadow-lg" :style="`top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`" x-transition:enter="transition ease-out duration-200" x-transition:enter-start="opacity-0 translate-y-1" x-transition:enter-end="opacity-100 translate-y-0">
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
                    <button @mouseenter="clearTimeout(closeTimer); activeNav = 'components'" @mouseleave="closeTimer = setTimeout(() => activeNav = null, 150)" class="group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none">
                        Components
                        <svg class="relative top-[1px] ml-1 h-3 w-3 transition duration-200" :class="activeNav === 'components' ? 'rotate-180' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
                    </button>
                    <template x-teleport="body">
                        <div x-show="activeNav === 'components'" @mouseenter="clearTimeout(closeTimer); activeNav = 'components'" @mouseleave="closeTimer = setTimeout(() => activeNav = null, 150)" x-cloak class="fixed left-1/2 z-50 w-[500px] -translate-x-1/2 rounded-md border bg-popover p-4 text-popover-foreground shadow-lg" :style="`top: ${$el.previousElementSibling?.getBoundingClientRect().bottom + 4}px;`" x-transition:enter="transition ease-out duration-200" x-transition:enter-start="opacity-0 translate-y-1" x-transition:enter-end="opacity-100 translate-y-0">
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
            <div class="flex h-[200px] w-full max-w-lg rounded-lg border overflow-hidden"
                x-data="{
                    dragging: false,
                    startX: 0,
                    startWidth: 0,
                    leftWidth: 50,
                    startDrag(e) {
                        this.dragging = true;
                        this.startX = e.clientX;
                        this.startWidth = this.leftWidth;
                        document.body.style.cursor = 'col-resize';
                        document.body.style.userSelect = 'none';
                    },
                    onDrag(e) {
                        if (!this.dragging) return;
                        const container = this.$refs.container;
                        const delta = e.clientX - this.startX;
                        const deltaPercent = (delta / container.offsetWidth) * 100;
                        this.leftWidth = Math.max(10, Math.min(90, this.startWidth + deltaPercent));
                    },
                    stopDrag() {
                        this.dragging = false;
                        document.body.style.cursor = '';
                        document.body.style.userSelect = '';
                    }
                }"
                x-ref="container"
                @mousemove.window="onDrag($event)"
                @mouseup.window="stopDrag()"
            >
                <div class="p-4 flex items-center justify-center bg-muted/30" :style="'width: ' + leftWidth + '%'">
                    <span class="text-sm font-medium" x-text="Math.round(leftWidth) + '%'"></span>
                </div>
                <div class="w-px bg-border cursor-col-resize hover:bg-primary/50 flex items-center justify-center relative group"
                    @mousedown.prevent="startDrag($event)"
                    tabindex="0"
                    @keydown.left.prevent="leftWidth = Math.max(10, leftWidth - 1)"
                    @keydown.right.prevent="leftWidth = Math.min(90, leftWidth + 1)"
                >
                    <div class="absolute z-10 h-8 w-3 rounded-sm border bg-border flex items-center justify-center group-hover:bg-primary/20 group-focus:ring-1 group-focus:ring-ring">
                        <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="h-2.5 w-2.5">
                            <circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/>
                        </svg>
                    </div>
                </div>
                <div class="flex-1 p-4 flex items-center justify-center bg-muted/10">
                    <span class="text-sm font-medium" x-text="Math.round(100 - leftWidth) + '%'"></span>
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
            <div x-data="{
                open: false,
                hour: 9,
                minute: 0,
                second: 0,
                period: 'AM',
                use12h: true,
                showSeconds: false,
                pos: { x: 0, y: 0 },
                flipTop: false,
                get maxHour() { return this.use12h ? 12 : 23; },
                get minHour() { return this.use12h ? 1 : 0; },
                get displayHour() { return this.hour.toString().padStart(2, '0'); },
                get displayMinute() { return this.minute.toString().padStart(2, '0'); },
                get displaySecond() { return this.second.toString().padStart(2, '0'); },
                get displayTime() {
                    let t = this.displayHour + ':' + this.displayMinute;
                    if (this.showSeconds) t += ':' + this.displaySecond;
                    if (this.use12h) t += ' ' + this.period;
                    return t;
                },
                toggle12h() {
                    const was12h = this.use12h;
                    this.use12h = !this.use12h;
                    if (this.use12h && !was12h) {
                        this.period = this.hour >= 12 ? 'PM' : 'AM';
                        this.hour = this.hour % 12;
                        if (this.hour === 0) this.hour = 12;
                    } else if (!this.use12h && was12h) {
                        let h = this.hour % 12;
                        if (this.period === 'PM') h += 12;
                        if (h === 24) h = 0;
                        this.hour = h;
                    }
                },
                incHour() { this.hour = this.hour >= this.maxHour ? this.minHour : this.hour + 1; },
                decHour() { this.hour = this.hour <= this.minHour ? this.maxHour : this.hour - 1; },
                incMinute() { this.minute = this.minute >= 59 ? 0 : this.minute + 1; },
                decMinute() { this.minute = this.minute <= 0 ? 59 : this.minute - 1; },
                incSecond() { this.second = this.second >= 59 ? 0 : this.second + 1; },
                decSecond() { this.second = this.second <= 0 ? 59 : this.second - 1; },
                togglePeriod() { this.period = this.period === 'AM' ? 'PM' : 'AM'; },
                setHour(val) { const n = parseInt(val, 10); if (!isNaN(n)) this.hour = Math.max(this.minHour, Math.min(this.maxHour, n)); },
                setMinute(val) { const n = parseInt(val, 10); if (!isNaN(n)) this.minute = Math.max(0, Math.min(59, n)); },
                setSecond(val) { const n = parseInt(val, 10); if (!isNaN(n)) this.second = Math.max(0, Math.min(59, n)); },
                updatePos() {
                    const trigger = this.$refs.trigger;
                    if (!trigger) return;
                    const rect = trigger.getBoundingClientRect();
                    const height = 200;
                    const spaceBelow = window.innerHeight - rect.bottom;
                    const spaceAbove = rect.top;
                    this.flipTop = spaceBelow < height && spaceAbove > spaceBelow;
                    this.pos.x = rect.left;
                    this.pos.y = this.flipTop ? rect.top : rect.bottom;
                },
                onScroll() { if (this.open) this.updatePos(); },
                init() {
                    this._scrollHandler = () => this.onScroll();
                    window.addEventListener('scroll', this._scrollHandler, true);
                    window.addEventListener('resize', this._scrollHandler);
                },
                destroy() {
                    window.removeEventListener('scroll', this._scrollHandler, true);
                    window.removeEventListener('resize', this._scrollHandler);
                }
            }" x-init="init()" @destroy="destroy()" class="inline-block">
                <button x-ref="trigger" @click="open = !open; $nextTick(() => updatePos())" class="flex h-9 w-[140px] items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm hover:bg-accent">
                    <span x-text="displayTime"></span>
                    <svg class="h-4 w-4 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                </button>
                <template x-teleport="body">
                    <div x-show="open" @click.away="open = false" x-cloak class="fixed z-[9999] p-3 rounded-md border border-border bg-popover text-popover-foreground shadow-md" :style="`left: ${pos.x}px; ${flipTop ? 'bottom: ' + (window.innerHeight - pos.y + 8) + 'px' : 'top: ' + (pos.y + 8) + 'px'}`">
                        <div class="flex items-center justify-between mb-3 gap-2">
                            <button type="button" @click="toggle12h()" class="text-xs px-2 py-1 rounded border border-input hover:bg-accent" x-text="use12h ? '12h' : '24h'"></button>
                            <button type="button" @click="showSeconds = !showSeconds" class="text-xs px-2 py-1 rounded border border-input hover:bg-accent" x-text="showSeconds ? 'HH:MM:SS' : 'HH:MM'"></button>
                        </div>
                        <div class="flex items-center gap-1">
                            <div class="flex flex-col items-center">
                                <button type="button" @click="incHour()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                <input type="text" :value="displayHour" @blur="setHour($event.target.value); $event.target.value = displayHour" @keydown.up.prevent="incHour()" @keydown.down.prevent="decHour()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displayHour" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                <button type="button" @click="decHour()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                            </div>
                            <span class="text-lg font-bold">:</span>
                            <div class="flex flex-col items-center">
                                <button type="button" @click="incMinute()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                <input type="text" :value="displayMinute" @blur="setMinute($event.target.value); $event.target.value = displayMinute" @keydown.up.prevent="incMinute()" @keydown.down.prevent="decMinute()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displayMinute" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                <button type="button" @click="decMinute()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                            </div>
                            <template x-if="showSeconds">
                                <span class="text-lg font-bold">:</span>
                            </template>
                            <template x-if="showSeconds">
                                <div class="flex flex-col items-center">
                                    <button type="button" @click="incSecond()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                    <input type="text" :value="displaySecond" @blur="setSecond($event.target.value); $event.target.value = displaySecond" @keydown.up.prevent="incSecond()" @keydown.down.prevent="decSecond()" @keydown.enter="$event.target.blur()" @focus="$event.target.value = ''; $event.target.placeholder = displaySecond" class="w-10 h-8 text-lg font-mono text-center bg-transparent border rounded focus:outline-none focus:ring-1 focus:ring-ring" maxlength="2" />
                                    <button type="button" @click="decSecond()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                </div>
                            </template>
                            <template x-if="use12h">
                                <div class="flex flex-col items-center ml-2">
                                    <button type="button" @click="togglePeriod()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m18 15-6-6-6 6"/></svg></button>
                                    <span class="text-lg font-mono w-10 text-center" x-text="period"></span>
                                    <button type="button" @click="togglePeriod()" class="h-6 w-10 flex items-center justify-center rounded hover:bg-accent"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg></button>
                                </div>
                            </template>
                        </div>
                    </div>
                </template>
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
&lt;@button&gt;Click me&lt;/@button&gt;

// Variants
&lt;@button variant=button::Variant::Secondary&gt;Secondary&lt;/@button&gt;
&lt;@button variant=button::Variant::Destructive&gt;Delete&lt;/@button&gt;
&lt;@button variant=button::Variant::Outline&gt;Outline&lt;/@button&gt;
&lt;@button variant=button::Variant::Ghost&gt;Ghost&lt;/@button&gt;

// Sizes
&lt;@button size=button::Size::Sm&gt;Small&lt;/@button&gt;
&lt;@button size=button::Size::Lg&gt;Large&lt;/@button&gt;

// Disabled
&lt;@button disabled=true&gt;Disabled&lt;/@button&gt;"#.to_string(),

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
        &lt;@card::title&gt;Card Title&lt;/@card::title&gt;
        &lt;@card::description&gt;Card description&lt;/@card::description&gt;
    &lt;/@card::header&gt;
    &lt;@card::content&gt;
        Your content here
    &lt;/@card::content&gt;
    &lt;@card::footer&gt;
        &lt;@button&gt;Action&lt;/@button&gt;
    &lt;/@card::footer&gt;
&lt;/@card&gt;"#.to_string(),

        "checkbox" => r#"@import components/checkbox.wtz as checkbox

// Basic checkbox
&lt;@checkbox name="terms" /&gt;

// With label
&lt;div class="flex items-center gap-2"&gt;
    &lt;@checkbox id="accept" name="accept" /&gt;
    &lt;@label for_id="accept"&gt;Accept terms&lt;/@label&gt;
&lt;/div&gt;

// Checked by default
&lt;@checkbox name="newsletter" checked=true /&gt;"#.to_string(),

        "switch" => r#"@import components/switch.wtz as switch

// Basic switch
&lt;@switch name="notifications" /&gt;

// With label
&lt;div class="flex items-center gap-2"&gt;
    &lt;@switch id="airplane" name="airplane" /&gt;
    &lt;@label for_id="airplane"&gt;Airplane Mode&lt;/@label&gt;
&lt;/div&gt;

// Enabled by default
&lt;@switch name="wifi" checked=true /&gt;"#.to_string(),

        "badge" => r#"@import components/badge.wtz as badge

// Default badge
&lt;@badge&gt;Badge&lt;/@badge&gt;

// Variants
&lt;@badge variant=badge::Variant::Secondary&gt;Secondary&lt;/@badge&gt;
&lt;@badge variant=badge::Variant::Destructive&gt;Error&lt;/@badge&gt;
&lt;@badge variant=badge::Variant::Outline&gt;Outline&lt;/@badge&gt;"#.to_string(),

        "alert" => r#"@import components/alert.wtz as alert

// Default alert
&lt;@alert&gt;
    &lt;@alert::title&gt;Heads up!&lt;/@alert::title&gt;
    &lt;@alert::description&gt;
        You can add components using the CLI.
    &lt;/@alert::description&gt;
&lt;/@alert&gt;

// Destructive alert
&lt;@alert variant=alert::Variant::Destructive&gt;
    &lt;@alert::title&gt;Error&lt;/@alert::title&gt;
    &lt;@alert::description&gt;Something went wrong.&lt;/@alert::description&gt;
&lt;/@alert&gt;"#.to_string(),

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
&lt;@label&gt;Email&lt;/@label&gt;

// Associated with input
&lt;@label for_id="email"&gt;Email&lt;/@label&gt;
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
    &lt;@select::item value="apple"&gt;Apple&lt;/@select::item&gt;
    &lt;@select::item value="banana"&gt;Banana&lt;/@select::item&gt;
    &lt;@select::item value="orange"&gt;Orange&lt;/@select::item&gt;
&lt;/@select&gt;"#.to_string(),

        "tabs" => r#"@import components/tabs.wtz as tabs

&lt;@tabs default="account"&gt;
    &lt;@tabs::list&gt;
        &lt;@tabs::trigger value="account"&gt;Account&lt;/@tabs::trigger&gt;
        &lt;@tabs::trigger value="password"&gt;Password&lt;/@tabs::trigger&gt;
    &lt;/@tabs::list&gt;
    &lt;@tabs::content value="account"&gt;
        Account settings here
    &lt;/@tabs::content&gt;
    &lt;@tabs::content value="password"&gt;
        Password settings here
    &lt;/@tabs::content&gt;
&lt;/@tabs&gt;"#.to_string(),

        "accordion" => r#"@import components/accordion.wtz as accordion

&lt;@accordion&gt;
    &lt;@accordion::item value="item-1"&gt;
        &lt;@accordion::trigger&gt;Is it accessible?&lt;/@accordion::trigger&gt;
        &lt;@accordion::content&gt;
            Yes. It adheres to the WAI-ARIA pattern.
        &lt;/@accordion::content&gt;
    &lt;/@accordion::item&gt;
    &lt;@accordion::item value="item-2"&gt;
        &lt;@accordion::trigger&gt;Is it styled?&lt;/@accordion::trigger&gt;
        &lt;@accordion::content&gt;
            Yes. It comes with default styles.
        &lt;/@accordion::content&gt;
    &lt;/@accordion::item&gt;
&lt;/@accordion&gt;"#.to_string(),

        "dialog" => r#"@import components/dialog.wtz as dialog

&lt;@dialog&gt;
    &lt;@dialog::trigger&gt;
        &lt;@button&gt;Open Dialog&lt;/@button&gt;
    &lt;/@dialog::trigger&gt;
    &lt;@dialog::content&gt;
        &lt;@dialog::header&gt;
            &lt;@dialog::title&gt;Edit profile&lt;/@dialog::title&gt;
            &lt;@dialog::description&gt;
                Make changes to your profile here.
            &lt;/@dialog::description&gt;
        &lt;/@dialog::header&gt;
        &lt;@dialog::footer&gt;
            &lt;@button variant=button::Variant::Outline&gt;Cancel&lt;/@button&gt;
            &lt;@button&gt;Save&lt;/@button&gt;
        &lt;/@dialog::footer&gt;
    &lt;/@dialog::content&gt;
&lt;/@dialog&gt;"#.to_string(),

        "alert-dialog" => r#"@import components/alert_dialog.wtz as alert_dialog

&lt;@alert_dialog&gt;
    &lt;@alert_dialog::trigger&gt;
        &lt;@button variant=button::Variant::Destructive&gt;
            Delete Account
        &lt;/@button&gt;
    &lt;/@alert_dialog::trigger&gt;
    &lt;@alert_dialog::content&gt;
        &lt;@alert_dialog::header&gt;
            &lt;@alert_dialog::title&gt;Are you sure?&lt;/@alert_dialog::title&gt;
            &lt;@alert_dialog::description&gt;
                This action cannot be undone.
            &lt;/@alert_dialog::description&gt;
        &lt;/@alert_dialog::header&gt;
        &lt;@alert_dialog::footer&gt;
            &lt;@alert_dialog::cancel&gt;Cancel&lt;/@alert_dialog::cancel&gt;
            &lt;@alert_dialog::action&gt;Delete&lt;/@alert_dialog::action&gt;
        &lt;/@alert_dialog::footer&gt;
    &lt;/@alert_dialog::content&gt;
&lt;/@alert_dialog&gt;"#.to_string(),

        "dropdown" => r#"@import components/dropdown.wtz as dropdown

&lt;@dropdown&gt;
    &lt;@dropdown::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Open Menu&lt;/@button&gt;
    &lt;/@dropdown::trigger&gt;
    &lt;@dropdown::content&gt;
        &lt;@dropdown::label&gt;My Account&lt;/@dropdown::label&gt;
        &lt;@dropdown::separator /&gt;
        &lt;@dropdown::item&gt;Profile&lt;/@dropdown::item&gt;
        &lt;@dropdown::item&gt;Settings&lt;/@dropdown::item&gt;
        &lt;@dropdown::separator /&gt;
        &lt;@dropdown::item class="text-destructive"&gt;Log out&lt;/@dropdown::item&gt;
    &lt;/@dropdown::content&gt;
&lt;/@dropdown&gt;"#.to_string(),

        "popover" => r#"@import components/popover.wtz as popover

&lt;@popover&gt;
    &lt;@popover::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Open&lt;/@button&gt;
    &lt;/@popover::trigger&gt;
    &lt;@popover::content&gt;
        &lt;div class="grid gap-4"&gt;
            &lt;h4 class="font-medium"&gt;Dimensions&lt;/h4&gt;
            &lt;p class="text-sm text-muted-foreground"&gt;
                Set the dimensions for the layer.
            &lt;/p&gt;
        &lt;/div&gt;
    &lt;/@popover::content&gt;
&lt;/@popover&gt;"#.to_string(),

        "tooltip" => r#"@import components/tooltip.wtz as tooltip

&lt;@tooltip&gt;
    &lt;@tooltip::trigger&gt;
        &lt;@button variant=button::Variant::Outline&gt;Hover me&lt;/@button&gt;
    &lt;/@tooltip::trigger&gt;
    &lt;@tooltip::content&gt;
        Add to library
    &lt;/@tooltip::content&gt;
&lt;/@tooltip&gt;"#.to_string(),

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
