//! HTTP request handlers

use crate::flag::generate_flag_bytes;
use crate::types::{parse_colors, FlagConfig, FlagError, FlagQuery, OutputFormat, PrideFlagPreset};
use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{info, warn};

/// HTTP error response with JSON body
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

impl IntoResponse for FlagError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            FlagError::InvalidConfig(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            FlagError::SvgParse(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FlagError::BufferCreation { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            FlagError::ImageEncode { .. } => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = serde_json::to_string(&ErrorResponse {
            error: error_message,
            details: None,
        })
        .unwrap_or_else(|_| r#"{"error":"Internal server error"}"#.to_string());

        (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
    }
}

/// GET /flag - Generate pride flag with query parameters
///
/// Query Parameters:
/// - preset: Device preset (e.g., "desktop-4k", "iphone-14-pro-max")
/// - pride: Pride flag preset (rainbow, bear, bisexual, transgender, pansexual, lesbian, asexual, nonbinary, progress)
/// - width: Custom width in pixels (overrides preset)
/// - height: Custom height in pixels (overrides preset)
/// - format: Output format (png, jpeg, webp) - default: png
/// - colors: Custom colors as comma-separated hex (e.g., "FF0000,00FF00,0000FF") - overrides pride preset
/// - stripe_count: Number of stripes to draw (defaults to palette length)
/// - paw_size: Paw size ratio 0.01-1.0 - default: 0.35
/// - center_paw: Center the paw (true/false) - default: true
/// - transparent: Use transparent background (true/false) - default: false
/// - include_overlay: Include paw overlay (true/false) - default: auto based on flag type
#[tracing::instrument(skip_all, fields(
    width = ?query.width.or_else(|| query.preset.map(|p| p.into()).map(|(w, _)| w)),
    height = ?query.height.or_else(|| query.preset.map(|p| p.into()).map(|(_, h)| h)),
    format = ?query.format,
    pride = ?query.pride,
    custom_colors = ?query.colors.is_some()
))]
pub async fn generate_flag_handler(Query(query): Query<FlagQuery>) -> Result<Response, FlagError> {
    // Determine palette and stripe count
    let (palette, default_stripe_count, includes_overlay) = if let Some(colors_str) = &query.colors {
        // Custom colors take precedence
        let custom_palette = parse_colors(colors_str)?;
        let stripe_count = custom_palette.len() as u32;
        (custom_palette, stripe_count, false)
    } else if let Some(pride_preset) = query.pride {
        // Use pride preset palette
        let palette = pride_preset.palette().to_vec();
        let stripe_count = palette.len() as u32;
        (palette, stripe_count, pride_preset.includes_overlay())
    } else {
        // Default to bear flag for backward compatibility
        let palette = PrideFlagPreset::Bear.palette().to_vec();
        let stripe_count = palette.len() as u32;
        (palette, stripe_count, true)
    };

    // Determine stripe count (use custom if provided, otherwise use default)
    let stripe_count = query.stripe_count.unwrap_or(default_stripe_count);

    // Determine overlay setting
    let include_overlay = query.include_overlay.unwrap_or(includes_overlay);

    // Build configuration from query params
    let mut config = if let (Some(width), Some(height)) = (query.width, query.height) {
        FlagConfig {
            width,
            height,
            output_format: query.format,
            palette,
            stripe_count,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            transparent: query.transparent,
            include_overlay,
        }
    } else if let Some(preset) = query.preset {
        let mut config = FlagConfig::from_preset(preset);
        config.output_format = query.format;
        config.palette = palette;
        config.stripe_count = stripe_count;
        config.paw_size_ratio = query.paw_size;
        config.center_paw = query.center_paw;
        config.transparent = query.transparent;
        config.include_overlay = include_overlay;
        config
    } else {
        // Use default 4K dimensions
        FlagConfig {
            width: 3840,
            height: 2160,
            output_format: query.format,
            palette,
            stripe_count,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            transparent: query.transparent,
            include_overlay,
        }
    };

    // Warn and disable transparent for JPEG
    if config.transparent && matches!(config.output_format, OutputFormat::Jpeg) {
        warn!("JPEG does not support transparency, using opaque background");
        config.transparent = false;
    }

    info!(
        "Generating {}x{} {} flag with {} stripes in {:?} format",
        config.width,
        config.height,
        if query.pride.is_some() { "pride" } else { "custom" },
        config.stripe_count,
        config.output_format
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

#[cfg(test)]
mod tests {
    use crate::router::create_router;
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_default() {
        let response = create_router()
            .oneshot(Request::builder().uri("/flag").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/png"
        );
    }

    #[tokio::test]
    async fn test_flag_endpoint_with_preset() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?preset=iphone-14-pro-max")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_with_custom_dimensions() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_jpeg_format() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?format=jpeg&width=320&height=240")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let headers = response.headers().clone();

        if status != StatusCode::OK {
            let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            panic!("Expected 200 OK, got {}, body: {}", status, body_str);
        }

        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/jpeg");
    }

    #[tokio::test]
    async fn test_flag_endpoint_invalid_dimensions() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?width=0&height=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_flag_endpoint_rainbow_pride() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?pride=rainbow&width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_transgender_pride() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?pride=transgender&width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_custom_colors() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?colors=FF0000,00FF00,0000FF&width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_custom_stripe_count() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?pride=rainbow&stripe_count=12&width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_flag_endpoint_invalid_colors() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/flag?colors=INVALID&width=640&height=480")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
