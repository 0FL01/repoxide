//! Repomix Rust Server
//!
//! A high-performance Rust implementation of the repomix web server using Axum.

use axum::{
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

mod error;
mod handlers;
mod state;
mod types;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "repomix_server=debug,tower_http=debug".into()),
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

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Pack API
        .route("/api/pack", post(handlers::pack))
        // Upload API (Phase 4)
        .route("/api/upload/init", post(handlers::upload_init))
        .route("/api/upload/chunk", post(handlers::upload_chunk))
        .route("/api/upload/status/{id}", get(handlers::upload_status))
        // Share state with all routes
        .with_state(state)
        // Middleware layers (order matters: first added = outermost)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

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
