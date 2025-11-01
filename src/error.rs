//! Error types for flag generation operations

use thiserror::Error;

/// Errors that can occur during flag generation
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Failed to save image to {path}: {source}")]
    ImageSave {
        path: String,
        source: image::ImageError,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unsupported output format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to determine output format from path: {0}")]
    FormatDetection(String),
}
