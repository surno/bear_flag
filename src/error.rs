//! Error types and error handling
//!
//! Defines application-specific errors and their HTTP response conversions.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during flag generation
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Failed to encode image as {format:?}: {source}")]
    ImageEncode {
        format: crate::config::OutputFormat,
        source: image::ImageError,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

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
