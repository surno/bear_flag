//! Application router wiring and middleware.

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::routes::{generate_flag_handler, health_handler};

/// Creates the Axum router with all routes and middleware.
pub fn create_router() -> Router {
    Router::new()
        .route("/flag", get(generate_flag_handler))
        .route("/health", get(health_handler))
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
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
    async fn flag_endpoint_returns_png_by_default() {
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
    async fn flag_endpoint_accepts_preset() {
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
    async fn flag_endpoint_accepts_custom_dimensions() {
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
    async fn flag_endpoint_handles_jpeg_format() {
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
            panic!("Expected 200 OK, got {status}, body: {body_str}");
        }

        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/jpeg");
    }

    #[tokio::test]
    async fn flag_endpoint_rejects_invalid_dimensions() {
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
