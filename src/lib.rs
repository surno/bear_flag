//! Bear Flag Generator Library
//!
//! Generates high-quality gay bear pride flags with smooth color gradients
//! and bear paw overlays. The library provides both programmatic API
//! and can be used as a CLI tool.

pub mod config;
pub mod error;
pub mod format;
pub mod render;

pub use config::{DevicePreset, FlagConfig};
pub use error::FlagError;
pub use format::OutputFormat;
pub use render::generate_flag;
