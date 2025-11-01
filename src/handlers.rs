//! HTTP request handlers
//!
//! Implements the API endpoints for the bear flag service.

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::{info, instrument, warn};

use crate::config::OutputFormat;
use crate::error::FlagError;
use crate::image::flag::generate_flag_bytes;
use crate::query::FlagQuery;

/// GET /flag - Generate bear pride flag with query parameters
///
/// Query Parameters:
/// - preset: Device preset (e.g., "desktop-4k", "iphone-14-pro-max")
/// - width: Custom width in pixels (overrides preset)
/// - height: Custom height in pixels (overrides preset)
/// - format: Output format (png, jpeg, webp) - default: png
/// - paw_size: Paw size ratio 0.01-1.0 - default: 0.35
/// - center_paw: Center the paw (true/false) - default: true
/// - transparent: Use transparent background (true/false) - default: false
#[instrument(skip_all, fields(
    width = ?query.width.or_else(|| query.preset.map(|p| p.into()).map(|(w, _)| w)),
    height = ?query.height.or_else(|| query.preset.map(|p| p.into()).map(|(_, h)| h)),
    format = ?query.format
))]
pub async fn generate_flag_handler(Query(query): Query<FlagQuery>) -> Result<Response, FlagError> {
    // Build configuration from query params
    let mut config = query.to_config();

    // Warn and disable transparent for JPEG
    if config.transparent && matches!(config.output_format, OutputFormat::Jpeg) {
        warn!("JPEG does not support transparency, using opaque background");
        config.transparent = false;
    }

    info!(
        "Generating {}x{} flag in {:?} format",
        config.width, config.height, config.output_format
    );

    let bytes = generate_flag_bytes(&config)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, config.output_format.mime_type())],
        bytes,
    )
        .into_response())
}

/// GET /health - Health check endpoint
pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
