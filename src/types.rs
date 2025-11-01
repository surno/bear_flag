//! Type definitions for the Bear Flag API service

use image::ImageFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Preset device configurations with appropriate dimensions for wallpapers
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum DevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    #[serde(rename = "iphone-14-pro-max")]
    IPhone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    #[serde(rename = "iphone-14-pro")]
    IPhone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    #[serde(rename = "iphone-14")]
    IPhone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    #[serde(rename = "iphone-se")]
    IPhoneSE,
    /// iPad Pro 12.9" - 2732 x 2048 (landscape)
    #[serde(rename = "ipad-pro-12.9")]
    IPadPro129,
    /// iPad Pro 11" - 2388 x 1668 (landscape)
    #[serde(rename = "ipad-pro-11")]
    IPadPro11,
    /// iPad Air 10.9" - 2360 x 1640 (landscape)
    #[serde(rename = "ipad-air")]
    IPadAir,
    /// Android QHD - 2560 x 1440
    #[serde(rename = "android-qhd")]
    AndroidQHD,
    /// Android FHD - 1920 x 1080
    #[serde(rename = "android-fhd")]
    AndroidFHD,
    /// Desktop 4K - 3840 x 2160
    #[serde(rename = "desktop-4k")]
    Desktop4K,
    /// Desktop 1440p - 2560 x 1440
    #[serde(rename = "desktop-1440p")]
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    #[serde(rename = "desktop-1080p")]
    Desktop1080p,
}

impl From<DevicePreset> for (u32, u32) {
    /// Converts a device preset to (width, height) dimensions
    fn from(preset: DevicePreset) -> Self {
        match preset {
            DevicePreset::IPhone14ProMax => (2796, 1290),
            DevicePreset::IPhone14Pro => (2556, 1179),
            DevicePreset::IPhone14 => (2532, 1170),
            DevicePreset::IPhoneSE => (1334, 750),
            DevicePreset::IPadPro129 => (2732, 2048),
            DevicePreset::IPadPro11 => (2388, 1668),
            DevicePreset::IPadAir => (2360, 1640),
            DevicePreset::AndroidQHD => (2560, 1440),
            DevicePreset::AndroidFHD => (1920, 1080),
            DevicePreset::Desktop4K => (3840, 2160),
            DevicePreset::Desktop1440p => (2560, 1440),
            DevicePreset::Desktop1080p => (1920, 1080),
        }
    }
}

impl DevicePreset {
    /// Returns a human-readable name for the device preset
    pub fn display_name(self) -> &'static str {
        match self {
            DevicePreset::IPhone14ProMax => "iPhone 14/13/12 Pro Max",
            DevicePreset::IPhone14Pro => "iPhone 14/13/12 Pro",
            DevicePreset::IPhone14 => "iPhone 14/13/12",
            DevicePreset::IPhoneSE => "iPhone SE (3rd gen)",
            DevicePreset::IPadPro129 => "iPad Pro 12.9\"",
            DevicePreset::IPadPro11 => "iPad Pro 11\"",
            DevicePreset::IPadAir => "iPad Air 10.9\"",
            DevicePreset::AndroidQHD => "Android QHD",
            DevicePreset::AndroidFHD => "Android FHD",
            DevicePreset::Desktop4K => "Desktop 4K",
            DevicePreset::Desktop1440p => "Desktop 1440p",
            DevicePreset::Desktop1080p => "Desktop 1080p",
        }
    }
}

/// Output image format
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// PNG format (supports transparency)
    Png,
    /// JPEG format (no transparency support)
    #[serde(alias = "jpg")]
    Jpeg,
    /// WebP format (supports transparency)
    WebP,
}

impl From<OutputFormat> for ImageFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::WebP => ImageFormat::WebP,
        }
    }
}

impl OutputFormat {
    /// Returns the MIME type for this format
    pub fn mime_type(self) -> &'static str {
        match self {
            OutputFormat::Png => "image/png",
            OutputFormat::Jpeg => "image/jpeg",
            OutputFormat::WebP => "image/webp",
        }
    }
}

/// Pride flag presets with their standard color palettes
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum PrideFlagPreset {
    /// Traditional rainbow pride flag - 6 stripes
    #[serde(rename = "rainbow")]
    Rainbow,
    /// Bear pride flag - 14 warm brown/black stripes
    #[serde(rename = "bear")]
    Bear,
    /// Bisexual pride flag - 3 stripes (pink, purple, blue)
    #[serde(rename = "bisexual")]
    Bisexual,
    /// Transgender pride flag - 5 stripes (light blue, pink, white, pink, light blue)
    #[serde(rename = "transgender")]
    Transgender,
    /// Pansexual pride flag - 3 stripes (pink, yellow, blue)
    #[serde(rename = "pansexual")]
    Pansexual,
    /// Lesbian pride flag - 7 stripes (dark orange to white)
    #[serde(rename = "lesbian")]
    Lesbian,
    /// Asexual pride flag - 4 stripes (black, gray, white, purple)
    #[serde(rename = "asexual")]
    Asexual,
    /// Non-binary pride flag - 4 stripes (yellow, white, purple, black)
    #[serde(rename = "nonbinary")]
    NonBinary,
    /// Progress pride flag - rainbow with triangle (chevron) of additional colors
    #[serde(rename = "progress")]
    Progress,
}

impl PrideFlagPreset {
    /// Returns the color palette for this pride flag as u32 hex values (0xRRGGBB)
    pub fn palette(self) -> &'static [u32] {
        match self {
            PrideFlagPreset::Rainbow => &[0xE40303, 0xFF8C00, 0xFFED00, 0x008026, 0x004DFF, 0x750787],
            PrideFlagPreset::Bear => &[
                0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41,
                0x89491D, 0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
            ],
            PrideFlagPreset::Bisexual => &[0xD60270, 0xD60270, 0x9B4F96, 0x0038A8, 0x0038A8],
            PrideFlagPreset::Transgender => &[0x5BCEFA, 0xF5A9B8, 0xFFFFFF, 0xF5A9B8, 0x5BCEFA],
            PrideFlagPreset::Pansexual => &[0xFF218C, 0xFFD800, 0x21B1FF],
            PrideFlagPreset::Lesbian => &[
                0xD52D00, 0xEF7627, 0xFF9A56, 0xFFFFFF, 0xD162A4, 0xB55690, 0xA30262,
            ],
            PrideFlagPreset::Asexual => &[0x000000, 0xA3A3A3, 0xFFFFFF, 0x810081],
            PrideFlagPreset::NonBinary => &[0xFFF430, 0xFFFFFF, 0x9C59D1, 0x000000],
            PrideFlagPreset::Progress => &[0x000000, 0x784F17, 0xFFFFFF, 0x5BCEFA, 0xFFFFFF, 0xF5A9B8, 0x000000],
        }
    }

    /// Returns whether this flag type typically includes an overlay (like bear paw)
    pub fn includes_overlay(self) -> bool {
        matches!(self, PrideFlagPreset::Bear)
    }
}

/// Errors that can occur during flag generation
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Failed to encode image as {format:?}: {source}")]
    ImageEncode {
        format: OutputFormat,
        source: image::ImageError,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Configuration for flag generation
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Output image width in pixels
    pub width: u32,
    /// Output image height in pixels
    pub height: u32,
    /// Image format for output
    pub output_format: OutputFormat,
    /// Color palette as u32 hex values (0xRRGGBB)
    pub palette: Vec<u32>,
    /// Number of stripes to draw
    pub stripe_count: u32,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
    /// Whether to use transparent background (only for formats that support it)
    pub transparent: bool,
    /// Whether to include the paw overlay (only for bear flag)
    pub include_overlay: bool,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_format: OutputFormat::Png,
            palette: PrideFlagPreset::Bear.palette().to_vec(),
            stripe_count: PrideFlagPreset::Bear.palette().len() as u32,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
            include_overlay: true,
        }
    }
}

impl FlagConfig {
    /// Creates a configuration from a device preset
    ///
    /// Uses sensible defaults for paw sizing and positioning
    pub fn from_preset(preset: DevicePreset) -> Self {
        let (width, height) = preset.into();
        Self {
            width,
            height,
            output_format: OutputFormat::Png,
            palette: PrideFlagPreset::Bear.palette().to_vec(),
            stripe_count: PrideFlagPreset::Bear.palette().len() as u32,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
            include_overlay: true,
        }
    }

    /// Validates configuration parameters
    ///
    /// # Errors
    ///
    /// Returns `FlagError::InvalidConfig` if any parameters are invalid
    pub fn validate(&self) -> Result<(), FlagError> {
        if self.width == 0 || self.height == 0 {
            return Err(FlagError::InvalidConfig(
                "Width and height must be non-zero".to_string(),
            ));
        }
        if self.width > 10000 || self.height > 10000 {
            return Err(FlagError::InvalidConfig(
                "Width and height must not exceed 10000 pixels".to_string(),
            ));
        }
        if !(0.01..=1.0).contains(&self.paw_size_ratio) {
            return Err(FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }
        if self.palette.is_empty() {
            return Err(FlagError::InvalidConfig(
                "Color palette must not be empty".to_string(),
            ));
        }
        if self.stripe_count == 0 {
            return Err(FlagError::InvalidConfig(
                "Stripe count must be non-zero".to_string(),
            ));
        }
        if self.stripe_count > 100 {
            return Err(FlagError::InvalidConfig(
                "Stripe count must not exceed 100".to_string(),
            ));
        }
        Ok(())
    }
}

/// Query parameters for flag generation endpoint
#[derive(Debug, Deserialize)]
pub struct FlagQuery {
    /// Device preset for standard dimensions
    pub preset: Option<DevicePreset>,
    /// Pride flag preset (rainbow, bear, bisexual, transgender, etc.)
    pub pride: Option<PrideFlagPreset>,
    /// Custom width in pixels (overrides preset)
    pub width: Option<u32>,
    /// Custom height in pixels (overrides preset)
    pub height: Option<u32>,
    /// Output format (png, jpeg, webp)
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Custom colors as comma-separated hex values (e.g., "FF0000,00FF00,0000FF")
    /// Overrides pride preset if provided
    pub colors: Option<String>,
    /// Number of stripes to draw (defaults to palette length)
    pub stripe_count: Option<u32>,
    /// Paw size as fraction of height (0.01-1.0)
    #[serde(default = "default_paw_size")]
    pub paw_size: f32,
    /// Whether to center the paw (default: true)
    #[serde(default = "default_center_paw")]
    pub center_paw: bool,
    /// Whether to use transparent background (default: false)
    #[serde(default)]
    pub transparent: bool,
    /// Whether to include overlay (paw) on flag (default: auto based on flag type)
    pub include_overlay: Option<bool>,
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

/// Parses comma-separated hex color strings into u32 values
///
/// Accepts colors in formats: "FF0000", "#FF0000", "0xFF0000", "ff0000"
/// Returns an error if any color cannot be parsed
///
/// # Errors
///
/// Returns `FlagError::InvalidConfig` if any color is invalid
pub fn parse_colors(colors_str: &str) -> Result<Vec<u32>, FlagError> {
    let mut palette = Vec::new();

    for color_str in colors_str.split(',') {
        let color_str = color_str.trim();
        if color_str.is_empty() {
            continue;
        }

        // Remove optional prefix markers
        let color_str = color_str
            .strip_prefix('#')
            .or_else(|| color_str.strip_prefix("0x"))
            .or_else(|| color_str.strip_prefix("0X"))
            .unwrap_or(color_str);

        // Parse hex string to u32
        let color = u32::from_str_radix(color_str, 16)
            .map_err(|_| {
                FlagError::InvalidConfig(format!(
                    "Invalid hex color: '{}'. Expected format: RRGGBB or #RRGGBB",
                    color_str
                ))
            })?;

        // Validate it's a 6-digit hex color (0x000000 - 0xFFFFFF)
        if color > 0xFFFFFF {
            return Err(FlagError::InvalidConfig(format!(
                "Color value too large: 0x{:X}. Must be in range 0x000000-0xFFFFFF",
                color
            )));
        }

        palette.push(color);
    }

    if palette.is_empty() {
        return Err(FlagError::InvalidConfig(
            "At least one color must be provided".to_string(),
        ));
    }

    Ok(palette)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_zero_dimensions() {
        let config = FlagConfig {
            width: 0,
            height: 100,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_too_large() {
        let config = FlagConfig {
            width: 20000,
            height: 100,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_paw_ratio() {
        let config = FlagConfig {
            paw_size_ratio: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = FlagConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_device_preset_iphone_14_pro_max() {
        let (width, height) = DevicePreset::IPhone14ProMax.into();
        assert_eq!(width, 2796);
        assert_eq!(height, 1290);
    }

    #[test]
    fn test_device_preset_desktop_4k() {
        let (width, height) = DevicePreset::Desktop4K.into();
        assert_eq!(width, 3840);
        assert_eq!(height, 2160);
    }

    #[test]
    fn test_flag_config_from_preset() {
        let config = FlagConfig::from_preset(DevicePreset::IPhone14ProMax);
        assert_eq!(config.width, 2796);
        assert_eq!(config.height, 1290);
        assert_eq!(config.paw_size_ratio, 0.35);
        assert!(config.center_paw);
    }

    #[test]
    fn test_output_format_mime_types() {
        assert_eq!(OutputFormat::Png.mime_type(), "image/png");
        assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(OutputFormat::WebP.mime_type(), "image/webp");
    }

    #[test]
    fn test_parse_colors_simple() {
        let result = parse_colors("FF0000,00FF00,0000FF").unwrap();
        assert_eq!(result, vec![0xFF0000, 0x00FF00, 0x0000FF]);
    }

    #[test]
    fn test_parse_colors_with_hash() {
        let result = parse_colors("#FF0000,#00FF00,#0000FF").unwrap();
        assert_eq!(result, vec![0xFF0000, 0x00FF00, 0x0000FF]);
    }

    #[test]
    fn test_parse_colors_mixed_formats() {
        let result = parse_colors("FF0000,#00FF00,0x0000FF").unwrap();
        assert_eq!(result, vec![0xFF0000, 0x00FF00, 0x0000FF]);
    }

    #[test]
    fn test_parse_colors_lowercase() {
        let result = parse_colors("ff0000,00ff00,0000ff").unwrap();
        assert_eq!(result, vec![0xFF0000, 0x00FF00, 0x0000FF]);
    }

    #[test]
    fn test_parse_colors_invalid() {
        assert!(parse_colors("GGGGGG").is_err());
        assert!(parse_colors("").is_err());
        assert!(parse_colors("FF0000,INVALID").is_err());
    }

    #[test]
    fn test_pride_flag_palettes() {
        assert_eq!(PrideFlagPreset::Rainbow.palette().len(), 6);
        assert_eq!(PrideFlagPreset::Bear.palette().len(), 14);
        assert_eq!(PrideFlagPreset::Bisexual.palette().len(), 5);
        assert_eq!(PrideFlagPreset::Transgender.palette().len(), 5);
        assert_eq!(PrideFlagPreset::Pansexual.palette().len(), 3);
    }

    #[test]
    fn test_pride_flag_overlay() {
        assert!(PrideFlagPreset::Bear.includes_overlay());
        assert!(!PrideFlagPreset::Rainbow.includes_overlay());
        assert!(!PrideFlagPreset::Transgender.includes_overlay());
    }

    #[test]
    fn test_flag_config_with_custom_palette() {
        let config = FlagConfig {
            width: 1920,
            height: 1080,
            output_format: OutputFormat::Png,
            palette: vec![0xFF0000, 0x00FF00, 0x0000FF],
            stripe_count: 3,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: false,
            include_overlay: false,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_flag_config_validation_empty_palette() {
        let mut config = FlagConfig::default();
        config.palette.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_flag_config_validation_zero_stripes() {
        let mut config = FlagConfig::default();
        config.stripe_count = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_flag_config_validation_too_many_stripes() {
        let mut config = FlagConfig::default();
        config.stripe_count = 101;
        assert!(config.validate().is_err());
    }
}
