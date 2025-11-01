//! Core library for the Bear Flag API service.
//!
//! Exposes modules for application wiring, configuration, error handling,
//! rendering utilities, HTTP routes, telemetry, and server lifecycle
//! management.

pub mod app;
pub mod config;
pub mod error;
pub mod rendering;
pub mod routes;
pub mod server;
pub mod telemetry;

pub use app::create_router;
pub use config::{DevicePreset, FlagConfig, OutputFormat};
pub use error::{FlagError, ServerError};
pub use rendering::generate_flag_bytes;
pub use server::DEFAULT_BIND_ADDR;
