//! HTTP request handlers

use crate::flag::generate_flag_bytes;
use crate::types::{FlagConfig, FlagError, FlagQuery, OutputFormat};
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
/// - flag_preset: Pride flag preset (e.g., "rainbow", "bear", "trans", "bi", "pan")
/// - colors: Custom colors as comma-separated hex (e.g., "FF0000,00FF00,0000FF") - overrides flag_preset
/// - stripe_count: Number of stripes (1-50) - repeats/truncates colors if needed
/// - preset: Device preset (e.g., "desktop-4k", "iphone-14-pro-max")
/// - width: Custom width in pixels (overrides preset)
/// - height: Custom height in pixels (overrides preset)
/// - format: Output format (png, jpeg, webp) - default: png
/// - paw_size: Paw size ratio 0.01-1.0 - default: 0.35
/// - center_paw: Center the paw (true/false) - default: true
/// - include_paw: Include bear paw overlay (true/false) - default: auto-based on flag_preset
/// - transparent: Use transparent background (true/false) - default: false
#[tracing::instrument(skip_all, fields(
    flag_preset = ?query.flag_preset,
    width = ?query.width.or_else(|| query.preset.map(|p| p.into()).map(|(w, _)| w)),
    height = ?query.height.or_else(|| query.preset.map(|p| p.into()).map(|(_, h)| h)),
    format = ?query.format
))]
pub async fn generate_flag_handler(Query(query): Query<FlagQuery>) -> Result<Response, FlagError> {
    // Resolve colors from query parameters
    let colors = query.resolve_colors()?;
    let include_paw = query.should_include_paw();

    // Build configuration from query params
    let mut config = if let (Some(width), Some(height)) = (query.width, query.height) {
        FlagConfig {
            width,
            height,
            output_format: query.format,
            colors,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            include_paw,
            transparent: query.transparent,
        }
    } else if let Some(preset) = query.preset {
        let mut config = FlagConfig::from_preset(preset);
        config.output_format = query.format;
        config.colors = colors;
        config.paw_size_ratio = query.paw_size;
        config.center_paw = query.center_paw;
        config.include_paw = include_paw;
        config.transparent = query.transparent;
        config
    } else {
        // Use default 4K dimensions
        FlagConfig {
            width: 3840,
            height: 2160,
            output_format: query.format,
            colors,
            paw_size_ratio: query.paw_size,
            center_paw: query.center_paw,
            include_paw,
            transparent: query.transparent,
        }
    };

    // Warn and disable transparent for JPEG
    if config.transparent && matches!(config.output_format, OutputFormat::Jpeg) {
        warn!("JPEG does not support transparency, using opaque background");
        config.transparent = false;
    }

    info!(
        "Generating {}x{} flag with {} stripes in {:?} format{}",
        config.width,
        config.height,
        config.colors.len(),
        config.output_format,
        if config.include_paw {
            " (with paw)"
        } else {
            ""
        }
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
}
