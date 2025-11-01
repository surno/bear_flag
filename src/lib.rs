//! Gay Bear Flag API Service
//!
//! Axum web service that generates high-quality gay bear pride flags with smooth
//! color gradients and a centered bear paw overlay. The flag combines the traditional
//! bear pride colors with proper alpha compositing for professional results.

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod rendering;

// Re-export commonly used types for convenience
pub use config::{FlagConfig, BEAR_PALETTE, SMOOTH_WIDTH};
pub use error::FlagError;
pub use models::{DevicePreset, FlagQuery, OutputFormat};
pub use rendering::generate_flag_bytes;

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

/// Creates the Axum router with all routes and middleware
///
/// Configures:
/// - `/flag` - Flag generation endpoint
/// - `/health` - Health check endpoint
/// - HTTP request tracing
pub fn create_router() -> Router {
    Router::new()
        .route("/flag", get(handlers::generate_flag_handler))
        .route("/health", get(handlers::health_handler))
        .layer(TraceLayer::new_for_http())
}
