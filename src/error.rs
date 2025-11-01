//! Error types for the Bear Flag API service.

use std::net::SocketAddr;

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during flag generation.
#[derive(Error, Debug)]
pub enum FlagError {
    /// The embedded SVG asset could not be parsed.
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    /// A target image buffer could not be allocated at the requested dimensions.
    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    /// The generated image could not be encoded to the requested output format.
    #[error("Failed to encode image as {format:?}: {source}")]
    ImageEncode {
        format: crate::config::OutputFormat,
        source: image::ImageError,
    },

    /// The supplied configuration parameters are invalid.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// HTTP error response with JSON body.
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

/// Errors that can occur while running the HTTP server.
#[derive(Debug, Error)]
pub enum ServerError {
    /// The provided bind address could not be parsed.
    #[error("failed to parse bind address `{address}`: {source}")]
    BindAddressParse {
        /// Raw address string that failed to parse.
        address: String,
        /// Underlying parse error.
        #[source]
        source: std::net::AddrParseError,
    },
    /// The TCP listener could not bind to the provided socket address.
    #[error("failed to bind to {address}: {source}")]
    Bind {
        /// Socket address that failed to bind.
        address: SocketAddr,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The Axum server encountered an unrecoverable runtime error.
    #[error("server error: {source}")]
    Serve {
        /// Downstream IO error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_error_into_response_uses_json() {
        let response = FlagError::InvalidConfig("bad".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }
}
