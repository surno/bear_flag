//! Type definitions for the Bear Flag API service

use image::ImageFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Pride flag preset types with their standard color palettes
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrideFlagPreset {
    /// Classic rainbow pride flag (6 colors)
    Rainbow,
    /// Traditional bear pride flag with bear paw overlay
    Bear,
    /// Transgender pride flag (5 colors)
    Trans,
    /// Bisexual pride flag (3 colors)
    Bi,
    /// Pansexual pride flag (3 colors)
    Pan,
    /// Non-binary pride flag (4 colors)
    NonBinary,
    /// Asexual pride flag (4 colors)
    Asexual,
    /// Lesbian pride flag (5 colors)
    Lesbian,
    /// Gay men's pride flag (5 colors)
    GayMen,
    /// Progress pride flag (includes trans colors and brown/black stripes)
    Progress,
    /// Philadelphia pride flag (includes brown and black stripes)
    Philadelphia,
}

impl PrideFlagPreset {
    /// Returns the color palette for this flag preset as hex RGB values (0xRRGGBB)
    pub fn colors(&self) -> &'static [u32] {
        match self {
            PrideFlagPreset::Rainbow => &RAINBOW_PALETTE,
            PrideFlagPreset::Bear => &BEAR_PALETTE,
            PrideFlagPreset::Trans => &TRANS_PALETTE,
            PrideFlagPreset::Bi => &BI_PALETTE,
            PrideFlagPreset::Pan => &PAN_PALETTE,
            PrideFlagPreset::NonBinary => &NON_BINARY_PALETTE,
            PrideFlagPreset::Asexual => &ASEXUAL_PALETTE,
            PrideFlagPreset::Lesbian => &LESBIAN_PALETTE,
            PrideFlagPreset::GayMen => &GAY_MEN_PALETTE,
            PrideFlagPreset::Progress => &PROGRESS_PALETTE,
            PrideFlagPreset::Philadelphia => &PHILADELPHIA_PALETTE,
        }
    }

    /// Returns whether this flag preset should include the bear paw overlay
    pub fn includes_paw(&self) -> bool {
        matches!(self, PrideFlagPreset::Bear)
    }

    /// Returns a human-readable name for the flag preset
    #[allow(dead_code)]
    pub fn display_name(self) -> &'static str {
        match self {
            PrideFlagPreset::Rainbow => "Rainbow Pride",
            PrideFlagPreset::Bear => "Bear Pride",
            PrideFlagPreset::Trans => "Transgender Pride",
            PrideFlagPreset::Bi => "Bisexual Pride",
            PrideFlagPreset::Pan => "Pansexual Pride",
            PrideFlagPreset::NonBinary => "Non-Binary Pride",
            PrideFlagPreset::Asexual => "Asexual Pride",
            PrideFlagPreset::Lesbian => "Lesbian Pride",
            PrideFlagPreset::GayMen => "Gay Men's Pride",
            PrideFlagPreset::Progress => "Progress Pride",
            PrideFlagPreset::Philadelphia => "Philadelphia Pride",
        }
    }
}

/// Classic 6-stripe rainbow pride flag colors (red, orange, yellow, green, blue, violet)
const RAINBOW_PALETTE: [u32; 6] = [
    0xFF0018, // Red
    0xFFA52C, // Orange
    0xFFFF41, // Yellow
    0x008018, // Green
    0x0000F9, // Blue
    0x86007D, // Violet
];

/// Bear pride flag colors (warm browns to deep browns/blacks)
pub const BEAR_PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

/// Transgender pride flag colors (light blue, pink, white, pink, light blue)
const TRANS_PALETTE: [u32; 5] = [
    0x5BCEFA, // Light blue
    0xF5A9B8, // Pink
    0xFFFFFF, // White
    0xF5A9B8, // Pink
    0x5BCEFA, // Light blue
];

/// Bisexual pride flag colors (pink, purple, blue)
const BI_PALETTE: [u32; 3] = [
    0xD70071, // Pink
    0x9C4E97, // Purple
    0x0035AA, // Blue
];

/// Pansexual pride flag colors (pink, yellow, cyan)
const PAN_PALETTE: [u32; 3] = [
    0xFF1B8D, // Pink
    0xFFD700, // Yellow
    0x1BB3FF, // Cyan
];

/// Non-binary pride flag colors (yellow, white, purple, black)
const NON_BINARY_PALETTE: [u32; 4] = [
    0xFFF430, // Yellow
    0xFFFFFF, // White
    0x9C59D1, // Purple
    0x2C2C2C, // Black
];

/// Asexual pride flag colors (black, grey, white, purple)
const ASEXUAL_PALETTE: [u32; 4] = [
    0x000000, // Black
    0xA4A4A4, // Grey
    0xFFFFFF, // White
    0x810081, // Purple
];

/// Lesbian pride flag colors (dark orange, orange, light orange, white, pink, dark pink, purple)
const LESBIAN_PALETTE: [u32; 5] = [
    0xD52D00, // Dark orange
    0xEF7627, // Orange
    0xFF9A56, // Light orange
    0xFFFFFF, // White
    0xD162A4, // Pink
];

/// Gay men's pride flag colors (green, teal, cyan, white, blue, purple, violet)
const GAY_MEN_PALETTE: [u32; 5] = [
    0x078D70, // Green
    0x26CEAA, // Teal
    0x98E8C1, // Cyan
    0xFFFFFF, // White
    0x7BADE2, // Blue
];

/// Progress pride flag (rainbow with chevron including trans colors, brown, black)
const PROGRESS_PALETTE: [u32; 11] = [
    0x000000, // Black
    0x784F17, // Brown
    0xFF0018, // Red
    0xFFA52C, // Orange
    0xFFFF41, // Yellow
    0x008018, // Green
    0x0000F9, // Blue
    0x86007D, // Violet
    0x5BCEFA, // Trans blue
    0xF5A9B8, // Trans pink
    0xFFFFFF, // Trans white
];

/// Philadelphia pride flag (rainbow with brown and black stripes)
const PHILADELPHIA_PALETTE: [u32; 8] = [
    0x000000, // Black
    0x784F17, // Brown
    0xFF0018, // Red
    0xFFA52C, // Orange
    0xFFFF41, // Yellow
    0x008018, // Green
    0x0000F9, // Blue
    0x86007D, // Violet
];

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
    /// Color palette as hex RGB values (0xRRGGBB)
    pub colors: Vec<u32>,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
    /// Whether to include the bear paw overlay
    pub include_paw: bool,
    /// Whether to use transparent background (only for formats that support it)
    pub transparent: bool,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_format: OutputFormat::Png,
            colors: BEAR_PALETTE.to_vec(),
            paw_size_ratio: 0.35,
            center_paw: true,
            include_paw: true,
            transparent: false,
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
            colors: BEAR_PALETTE.to_vec(),
            paw_size_ratio: 0.35,
            center_paw: true,
            include_paw: true,
            transparent: false,
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
        if self.colors.is_empty() {
            return Err(FlagError::InvalidConfig(
                "At least one color must be specified".to_string(),
            ));
        }
        if self.colors.len() > 50 {
            return Err(FlagError::InvalidConfig(
                "Maximum 50 colors allowed".to_string(),
            ));
        }
        if !(0.01..=1.0).contains(&self.paw_size_ratio) {
            return Err(FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Query parameters for flag generation endpoint
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FlagQuery {
    /// Pride flag preset (e.g., "rainbow", "bear", "trans", "bi", "pan")
    pub flag_preset: Option<PrideFlagPreset>,
    /// Custom colors as comma-separated hex values (e.g., "FF0000,00FF00,0000FF")
    /// Overrides flag_preset if provided
    pub colors: Option<String>,
    /// Number of stripes (overrides colors length if provided, repeats/truncates colors)
    pub stripe_count: Option<u32>,
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
    /// Whether to include the bear paw overlay (default: auto-based on flag_preset)
    pub include_paw: Option<bool>,
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

impl Default for FlagQuery {
    fn default() -> Self {
        Self {
            flag_preset: None,
            colors: None,
            stripe_count: None,
            preset: None,
            width: None,
            height: None,
            format: default_format(),
            paw_size: default_paw_size(),
            center_paw: default_center_paw(),
            include_paw: None,
            transparent: false,
        }
    }
}

impl FlagQuery {
    /// Parses colors from the query string into a vector of hex RGB values
    ///
    /// Accepts comma-separated hex values with or without '#' prefix (e.g., "FF0000,00FF00" or "#FF0000,#00FF00")
    ///
    /// # Errors
    ///
    /// Returns `FlagError::InvalidConfig` if any color cannot be parsed
    pub fn parse_colors(&self) -> Result<Vec<u32>, FlagError> {
        let colors_str = self.colors.as_ref().ok_or_else(|| {
            FlagError::InvalidConfig(
                "Colors parameter is required when not using flag_preset".to_string(),
            )
        })?;

        let mut parsed_colors = Vec::new();
        for color_str in colors_str.split(',') {
            let color_str = color_str.trim().trim_start_matches('#');
            if color_str.len() != 6 {
                return Err(FlagError::InvalidConfig(format!(
                    "Invalid color format: '{}'. Expected 6 hex digits (RRGGBB)",
                    color_str
                )));
            }
            let color = u32::from_str_radix(color_str, 16).map_err(|e| {
                FlagError::InvalidConfig(format!("Invalid hex color '{}': {}", color_str, e))
            })?;
            parsed_colors.push(color);
        }

        if parsed_colors.is_empty() {
            return Err(FlagError::InvalidConfig(
                "At least one color must be provided".to_string(),
            ));
        }

        Ok(parsed_colors)
    }

    /// Resolves the color palette from query parameters
    ///
    /// Priority: custom colors > flag_preset > default bear palette
    pub fn resolve_colors(&self) -> Result<Vec<u32>, FlagError> {
        let mut colors = if self.colors.is_some() {
            // Custom colors override preset
            self.parse_colors()?
        } else if let Some(preset) = self.flag_preset {
            // Use preset colors
            preset.colors().to_vec()
        } else {
            // Default to bear palette for backwards compatibility
            BEAR_PALETTE.to_vec()
        };

        // Adjust stripe count if requested
        if let Some(stripe_count) = self.stripe_count {
            if stripe_count == 0 || stripe_count > 50 {
                return Err(FlagError::InvalidConfig(
                    "Stripe count must be between 1 and 50".to_string(),
                ));
            }

            if colors.len() < stripe_count as usize {
                // Repeat colors to reach desired count
                let original_len = colors.len();
                let repeats_needed = (stripe_count as usize + original_len - 1) / original_len;
                colors = colors
                    .iter()
                    .cycle()
                    .take(original_len * repeats_needed)
                    .copied()
                    .collect();
            }
            // Truncate to desired count
            colors.truncate(stripe_count as usize);
        }

        Ok(colors)
    }

    /// Determines whether to include the paw overlay
    pub fn should_include_paw(&self) -> bool {
        if let Some(include_paw) = self.include_paw {
            return include_paw;
        }
        // Auto-detect based on flag preset
        self.flag_preset.map(|p| p.includes_paw()).unwrap_or(true) // Default to true for backwards compatibility
    }
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
    fn test_pride_flag_preset_colors() {
        let rainbow = PrideFlagPreset::Rainbow.colors();
        assert_eq!(rainbow.len(), 6);
        assert_eq!(rainbow[0], 0xFF0018); // Red

        let trans = PrideFlagPreset::Trans.colors();
        assert_eq!(trans.len(), 5);

        let bear = PrideFlagPreset::Bear.colors();
        assert_eq!(bear.len(), 14);
    }

    #[test]
    fn test_pride_flag_preset_includes_paw() {
        assert!(PrideFlagPreset::Bear.includes_paw());
        assert!(!PrideFlagPreset::Rainbow.includes_paw());
        assert!(!PrideFlagPreset::Trans.includes_paw());
    }

    #[test]
    fn test_flag_query_parse_colors() {
        let query = FlagQuery {
            colors: Some("FF0000,00FF00,0000FF".to_string()),
            ..Default::default()
        };
        let colors = query.parse_colors().unwrap();
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], 0xFF0000);
        assert_eq!(colors[1], 0x00FF00);
        assert_eq!(colors[2], 0x0000FF);
    }

    #[test]
    fn test_flag_query_parse_colors_with_hash() {
        let query = FlagQuery {
            colors: Some("#FF0000,#00FF00,#0000FF".to_string()),
            ..Default::default()
        };
        let colors = query.parse_colors().unwrap();
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], 0xFF0000);
    }

    #[test]
    fn test_flag_query_resolve_colors_preset() {
        let query = FlagQuery {
            flag_preset: Some(PrideFlagPreset::Rainbow),
            ..Default::default()
        };
        let colors = query.resolve_colors().unwrap();
        assert_eq!(colors.len(), 6);
        assert_eq!(colors, PrideFlagPreset::Rainbow.colors());
    }

    #[test]
    fn test_flag_query_resolve_colors_custom() {
        let query = FlagQuery {
            colors: Some("FF0000,00FF00".to_string()),
            flag_preset: Some(PrideFlagPreset::Rainbow),
            ..Default::default()
        };
        let colors = query.resolve_colors().unwrap();
        // Custom colors should override preset
        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0], 0xFF0000);
    }

    #[test]
    fn test_flag_query_resolve_colors_stripe_count() {
        let query = FlagQuery {
            colors: Some("FF0000,00FF00".to_string()),
            stripe_count: Some(6),
            ..Default::default()
        };
        let colors = query.resolve_colors().unwrap();
        // Should repeat colors to reach 6
        assert_eq!(colors.len(), 6);
        assert_eq!(colors[0], 0xFF0000);
        assert_eq!(colors[1], 0x00FF00);
        assert_eq!(colors[2], 0xFF0000); // Repeated
    }

    #[test]
    fn test_flag_query_should_include_paw() {
        let query_bear = FlagQuery {
            flag_preset: Some(PrideFlagPreset::Bear),
            ..Default::default()
        };
        assert!(query_bear.should_include_paw());

        let query_rainbow = FlagQuery {
            flag_preset: Some(PrideFlagPreset::Rainbow),
            ..Default::default()
        };
        assert!(!query_rainbow.should_include_paw());

        let query_override = FlagQuery {
            flag_preset: Some(PrideFlagPreset::Rainbow),
            include_paw: Some(true),
            ..Default::default()
        };
        assert!(query_override.should_include_paw());
    }

    #[test]
    fn test_config_validation_empty_colors() {
        let config = FlagConfig {
            colors: vec![],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_too_many_colors() {
        let config = FlagConfig {
            colors: vec![0xFF0000; 51],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
