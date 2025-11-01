//! Query parameter types for HTTP endpoints
//!
//! Handles deserialization and conversion of query parameters to configuration.

use serde::Deserialize;

use crate::config::{DevicePreset, OutputFormat};

/// Query parameters for flag generation endpoint
#[derive(Debug, Deserialize)]
pub struct FlagQuery {
    /// Device preset for standard dimensions
    pub preset: Option<DevicePreset>,
    /// Custom width in pixels (overrides preset)
    pub width: Option<u32>,
    /// Custom height in pixels (overrides preset)
    pub height: Option<u32>,
    /// Output format (png, jpeg, webp)
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Paw size as fraction of height (0.01-1.0)
    #[serde(default = "default_paw_size")]
    pub paw_size: f32,
    /// Whether to center the paw (default: true)
    #[serde(default = "default_center_paw")]
    pub center_paw: bool,
    /// Whether to use transparent background (default: false)
    #[serde(default)]
    pub transparent: bool,
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

impl FlagQuery {
    /// Converts query parameters to a FlagConfig
    ///
    /// Priority: custom width/height > preset > default dimensions
    pub fn to_config(&self) -> crate::config::FlagConfig {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            crate::config::FlagConfig {
                width,
                height,
                output_format: self.format,
                paw_size_ratio: self.paw_size,
                center_paw: self.center_paw,
                transparent: self.transparent,
            }
        } else if let Some(preset) = self.preset {
            let (width, height) = preset.into();
            crate::config::FlagConfig {
                width,
                height,
                output_format: self.format,
                paw_size_ratio: self.paw_size,
                center_paw: self.center_paw,
                transparent: self.transparent,
            }
        } else {
            // Use default 4K dimensions
            crate::config::FlagConfig {
                width: 3840,
                height: 2160,
                output_format: self.format,
                paw_size_ratio: self.paw_size,
                center_paw: self.center_paw,
                transparent: self.transparent,
            }
        }
    }
}
