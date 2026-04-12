//! Repomix Rust Server
//!
//! A high-performance Rust implementation of the repomix web server using Axum.

use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod error;
mod handlers;
mod state;
mod types;
mod views;

const MAX_MULTIPART_BODY_SIZE: usize = 200 * 1024 * 1024;
const MAX_CHUNK_BODY_SIZE: usize = 2 * 1024 * 1024;
const DEFAULT_CORS_ALLOW_ORIGINS: [&str; 4] = [
    "https://repomix.com",
    "https://www.repomix.com",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
];

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "repomix_server=debug,tower_http=debug".into()),
        )
        .json()
        .init();

    tracing::info!("Initializing repomix server...");

    // Create application state
    let state = Arc::new(state::AppState::new());

    // Spawn background task for cleanup
    let cleanup_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_state.cleanup_expired().await;
        }
    });

    let multipart_routes = Router::new()
        .route("/api/pack", post(handlers::pack))
        .route("/pack", post(handlers::pack_page))
        .layer(DefaultBodyLimit::max(MAX_MULTIPART_BODY_SIZE));

    let chunk_upload_routes = Router::new()
        .route("/api/upload/chunk", post(handlers::upload_chunk))
        .layer(DefaultBodyLimit::max(MAX_CHUNK_BODY_SIZE));

    // Build router
    let app = Router::new()
        // Web frontend
        .route("/", get(handlers::index))
        .route("/en", get(handlers::index))
        .route("/ru", get(handlers::index_ru))
        .route("/images/repomix-logo.svg", get(handlers::repomix_logo_svg))
        .route("/static/repomix-home.css", get(handlers::home_css))
        .route("/static/repomix-home.js", get(handlers::home_js))
        .route("/schemas/{*path}", get(handlers::schema_asset))
        // Health check
        .route("/health", get(handlers::health))
        // Upload API
        .route("/api/upload/init", post(handlers::upload_init))
        .route("/api/upload/status/{id}", get(handlers::upload_status))
        .route("/{*path}", get(handlers::site_fallback))
        .merge(chunk_upload_routes)
        .merge(multipart_routes)
        // Share state with all routes
        .with_state(state)
        // Middleware layers (order matters: first added = outermost)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let allow_origins = std::env::var("CORS_ALLOW_ORIGIN")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .filter_map(|origin| match HeaderValue::from_str(origin) {
                    Ok(origin) => Some(origin),
                    Err(error) => {
                        tracing::warn!(origin, %error, "Ignoring invalid CORS allow origin");
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_CORS_ALLOW_ORIGINS
                .iter()
                .map(|origin| HeaderValue::from_static(origin))
                .collect()
        });

    let app = app.layer(
        CorsLayer::new()
            .allow_origin(allow_origins)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any),
    );

    // Get port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server is ready to accept connections");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
