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

use generated::{Library, LIBRARIES};

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
        .route("/library/{id}/theme/{theme}", get(library_showcase_themed))
        .nest_service("/static", ServeDir::new("static"));

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Showcase server running at http://127.0.0.1:3000");
    tracing::info!("Discovered {} libraries", LIBRARIES.len());

    for lib in LIBRARIES {
        tracing::info!("  - {} v{} ({} components)", lib.name, lib.version, lib.component_count);
    }

    axum::serve(listener, app).await.expect("Server error");
}

/// Index page showing all discovered libraries
async fn index() -> impl IntoResponse {
    let libraries_html: String = LIBRARIES
        .iter()
        .map(|lib| {
            format!(
                r#"
                <a href="/library/{}" class="block p-6 bg-card rounded-lg border border-border hover:border-primary transition-colors">
                    <div class="flex items-center justify-between mb-2">
                        <h2 class="text-xl font-semibold">{}</h2>
                        <span class="text-sm text-muted-foreground">v{}</span>
                    </div>
                    <p class="text-muted-foreground mb-4">{}</p>
                    <div class="text-sm text-muted-foreground">
                        {} components
                    </div>
                </a>
                "#,
                lib.id, lib.name, lib.version, lib.description, lib.component_count
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
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        border: 'hsl(var(--border))',
                        input: 'hsl(var(--input))',
                        ring: 'hsl(var(--ring))',
                        background: 'hsl(var(--background))',
                        foreground: 'hsl(var(--foreground))',
                        primary: {{
                            DEFAULT: 'hsl(var(--primary))',
                            foreground: 'hsl(var(--primary-foreground))',
                        }},
                        secondary: {{
                            DEFAULT: 'hsl(var(--secondary))',
                            foreground: 'hsl(var(--secondary-foreground))',
                        }},
                        destructive: {{
                            DEFAULT: 'hsl(var(--destructive))',
                            foreground: 'hsl(var(--destructive-foreground))',
                        }},
                        muted: {{
                            DEFAULT: 'hsl(var(--muted))',
                            foreground: 'hsl(var(--muted-foreground))',
                        }},
                        accent: {{
                            DEFAULT: 'hsl(var(--accent))',
                            foreground: 'hsl(var(--accent-foreground))',
                        }},
                        popover: {{
                            DEFAULT: 'hsl(var(--popover))',
                            foreground: 'hsl(var(--popover-foreground))',
                        }},
                        card: {{
                            DEFAULT: 'hsl(var(--card))',
                            foreground: 'hsl(var(--card-foreground))',
                        }},
                    }},
                }},
            }},
        }}
    </script>
    <style>
        :root {{
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
        }}

        .dark {{
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
        }}

        body {{
            background-color: hsl(var(--background));
            color: hsl(var(--foreground));
        }}
    </style>
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
</head>
<body class="min-h-screen" x-data="{{ dark: true }}" x-init="dark = localStorage.getItem('theme') !== 'light'" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="container mx-auto px-4 py-8 max-w-4xl">
        <header class="flex items-center justify-between mb-12">
            <div>
                <h1 class="text-4xl font-bold mb-2">Waltzing Showcase</h1>
                <p class="text-muted-foreground">Explore template libraries for the Waltzing engine</p>
            </div>
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
        </header>

        <main class="grid gap-4">
            {}
        </main>

        <footer class="mt-12 pt-8 border-t border-border text-center text-sm text-muted-foreground">
            <p>Powered by <a href="https://github.com/awesomike/waltzing" class="underline hover:text-foreground">Waltzing</a></p>
        </footer>
    </div>
</body>
</html>"#,
        libraries_html
    );

    Html(html)
}

/// Library showcase page
async fn library_showcase(Path(id): Path<String>) -> impl IntoResponse {
    library_showcase_with_theme(&id, "default")
}

/// Library showcase page with specific theme
async fn library_showcase_themed(Path((id, theme)): Path<(String, String)>) -> impl IntoResponse {
    library_showcase_with_theme(&id, &theme)
}

fn library_showcase_with_theme(id: &str, theme: &str) -> impl IntoResponse {
    let library = LIBRARIES.iter().find(|lib| lib.id == id);

    match library {
        Some(lib) => {
            let theme_class = match theme {
                "light" => "",
                "dark" => "dark",
                _ => "dark", // default to dark
            };

            // Generate the showcase content from compiled templates
            let showcase_content = generate_showcase_for_library(lib);

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en" class="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Waltzing Showcase</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        border: 'hsl(var(--border))',
                        input: 'hsl(var(--input))',
                        ring: 'hsl(var(--ring))',
                        background: 'hsl(var(--background))',
                        foreground: 'hsl(var(--foreground))',
                        primary: {{
                            DEFAULT: 'hsl(var(--primary))',
                            foreground: 'hsl(var(--primary-foreground))',
                        }},
                        secondary: {{
                            DEFAULT: 'hsl(var(--secondary))',
                            foreground: 'hsl(var(--secondary-foreground))',
                        }},
                        destructive: {{
                            DEFAULT: 'hsl(var(--destructive))',
                            foreground: 'hsl(var(--destructive-foreground))',
                        }},
                        muted: {{
                            DEFAULT: 'hsl(var(--muted))',
                            foreground: 'hsl(var(--muted-foreground))',
                        }},
                        accent: {{
                            DEFAULT: 'hsl(var(--accent))',
                            foreground: 'hsl(var(--accent-foreground))',
                        }},
                        popover: {{
                            DEFAULT: 'hsl(var(--popover))',
                            foreground: 'hsl(var(--popover-foreground))',
                        }},
                        card: {{
                            DEFAULT: 'hsl(var(--card))',
                            foreground: 'hsl(var(--card-foreground))',
                        }},
                    }},
                }},
            }},
        }}
    </script>
    <style>
        :root {{
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
        }}

        .dark {{
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
        }}

        body {{
            background-color: hsl(var(--background));
            color: hsl(var(--foreground));
        }}

        [x-cloak] {{
            display: none !important;
        }}
    </style>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/focus@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/collapse@3.x.x/dist/cdn.min.js"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
</head>
<body class="min-h-screen" x-data="{{ dark: document.documentElement.classList.contains('dark') }}" x-effect="document.documentElement.classList.toggle('dark', dark); localStorage.setItem('theme', dark ? 'dark' : 'light')">
    <div class="container mx-auto px-4 py-8 max-w-6xl">
        <header class="flex items-center justify-between mb-8">
            <div class="flex items-center gap-4">
                <a href="/" class="text-muted-foreground hover:text-foreground transition-colors">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="m12 19-7-7 7-7"></path>
                        <path d="M19 12H5"></path>
                    </svg>
                </a>
                <div>
                    <h1 class="text-3xl font-bold">{}</h1>
                    <p class="text-muted-foreground">{} &middot; v{}</p>
                </div>
            </div>
            <div class="flex items-center gap-2">
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
            </div>
        </header>

        <main>
            {}
        </main>
    </div>
</body>
</html>"#,
                theme_class,
                lib.name,
                lib.name,
                lib.description,
                lib.version,
                showcase_content
            );

            Html(html)
        }
        None => Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Not Found</title></head>
<body>
    <h1>Library not found: {}</h1>
    <a href="/">Back to home</a>
</body>
</html>"#,
            id
        )),
    }
}

/// Generate showcase HTML for a library
/// This will call the compiled template functions
fn generate_showcase_for_library(lib: &Library) -> String {
    // For now, generate a placeholder
    // TODO: Call the actual compiled template showcase function
    format!(
        r#"
        <div class="text-center py-12">
            <p class="text-muted-foreground mb-4">Component showcase for {} coming soon...</p>
            <p class="text-sm text-muted-foreground">{} components available</p>
        </div>
        "#,
        lib.name, lib.component_count
    )
}
