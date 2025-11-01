//! HTTP route handlers for the Bear Flag API.

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::{
    config::{DevicePreset, FlagConfig, OutputFormat},
    error::FlagError,
    rendering::generate_flag_bytes,
};

/// Query parameters for the flag generation endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct FlagQuery {
    /// Device preset for standard dimensions.
    preset: Option<DevicePreset>,
    /// Custom width in pixels (overrides preset).
    width: Option<u32>,
    /// Custom height in pixels (overrides preset).
    height: Option<u32>,
    /// Output format (png, jpeg, webp).
    #[serde(default = "default_format")]
    format: OutputFormat,
    /// Paw size as fraction of height (0.01-1.0).
    #[serde(default = "default_paw_size")]
    paw_size: f32,
    /// Whether to center the paw (default: true).
    #[serde(default = "default_center_paw")]
    center_paw: bool,
    /// Whether to use transparent background (default: false).
    #[serde(default)]
    transparent: bool,
}

fn default_format() -> OutputFormat {
    OutputFormat::Png
}

fn default_paw_size() -> f32 {
    0.35
}

fn default_center_paw() -> bool {
    true
}

/// GET /flag - Generate bear pride flag with query parameters.
///
/// Query Parameters:
/// - preset: Device preset (e.g., "desktop-4k", "iphone-14-pro-max")
/// - width: Custom width in pixels (overrides preset)
/// - height: Custom height in pixels (overrides preset)
/// - format: Output format (png, jpeg, webp) - default: png
/// - paw_size: Paw size ratio 0.01-1.0 - default: 0.35
/// - center_paw: Center the paw (true/false) - default: true
/// - transparent: Use transparent background (true/false) - default: false
#[tracing::instrument(skip_all, fields(
    width = ?query.width.or_else(|| query.preset.map(|p| p.into()).map(|(w, _)| w)),
    height = ?query.height.or_else(|| query.preset.map(|p| p.into()).map(|(_, h)| h)),
    format = ?query.format
))]
pub(crate) async fn generate_flag_handler(
    Query(query): Query<FlagQuery>,
) -> Result<Response, FlagError> {
    // Build configuration from query params.
    let mut config = if let (Some(width), Some(height)) = (query.width, query.height) {
        FlagConfig {
            width,
            height,
            output_format: query.format,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            transparent: query.transparent,
        }
    } else if let Some(preset) = query.preset {
        let (width, height) = preset.into();
        FlagConfig {
            width,
            height,
            output_format: query.format,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            transparent: query.transparent,
        }
    } else {
        // Use default 4K dimensions.
        FlagConfig {
            width: 3840,
            height: 2160,
            output_format: query.format,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            transparent: query.transparent,
        }
    };

    // Warn and disable transparent for JPEG.
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

/// GET /health - Health check endpoint.
pub(crate) async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Query as AxumQuery;

    #[tokio::test]
    async fn generate_flag_handler_rejects_invalid_dimensions() {
        let query = FlagQuery {
            preset: None,
            width: Some(0),
            height: Some(100),
            format: OutputFormat::Png,
            paw_size: 0.35,
            center_paw: true,
            transparent: false,
        };

        let result = generate_flag_handler(AxumQuery(query)).await;
        assert!(result.is_err());
    }
}
