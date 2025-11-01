//! Axum router configuration
//!
//! Sets up routes and middleware for the HTTP server.

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::handlers::{generate_flag_handler, health_handler};

/// Creates the Axum router with all routes and middleware
pub fn create_router() -> Router {
    Router::new()
        .route("/flag", get(generate_flag_handler))
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http())
}
