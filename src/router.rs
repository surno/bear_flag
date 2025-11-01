//! Axum router configuration

use crate::handlers::{generate_flag_handler, health_handler};
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

/// Creates the Axum router with all routes and middleware
pub fn create_router() -> Router {
    Router::new()
        .route("/flag", get(generate_flag_handler))
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http())
}
