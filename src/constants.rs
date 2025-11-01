//! Constants for the Bear Flag API service

/// Embeds assets/bear_paw.svg directly into the binary
pub const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

/// Number of pixels over which adjacent color stripes smoothly blend
pub const SMOOTH_WIDTH: u32 = 16;

/// Default server bind address
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
