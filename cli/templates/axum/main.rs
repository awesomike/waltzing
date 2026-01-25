//! {{project_name}} - Built with Waltzing
//!
//! Run with: cargo run
//! Development with hot reload: cargo watch -x run

use axum::{
    Router,
    response::Html,
    routing::get,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use waltzing::Template;

// Import the generated template module
mod templates;
use templates::index::IndexTemplate;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build application with routes
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Starting server at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<String> {
    let template = IndexTemplate {
        title: "Welcome to Waltzing".into(),
        message: "Your project is ready!".into(),
    };
    Html(template.render())
}
