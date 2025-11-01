//! Type definitions for the Bear Flag API service

use image::ImageFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Pride flag presets with their traditional color palettes
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrideFlagPreset {
    /// Traditional 6-stripe rainbow pride flag
    Rainbow,
    /// Gilbert Baker's original 8-stripe rainbow
    RainbowGilbertBaker,
    /// Transgender pride flag (light blue, pink, white)
    Trans,
    /// Bisexual pride flag (pink, purple, blue)
    Bisexual,
    /// Pansexual pride flag (pink, yellow, blue)
    Pansexual,
    /// Lesbian pride flag (orange to pink gradient)
    Lesbian,
    /// Asexual pride flag (black, grey, white, purple)
    Asexual,
    /// Non-binary pride flag (yellow, white, purple, black)
    Nonbinary,
    /// Genderqueer pride flag (lavender, white, green)
    Genderqueer,
    /// Genderfluid pride flag (pink, white, purple, black, blue)
    Genderfluid,
    /// Aromantic pride flag (green, light green, white, grey, black)
    Aromantic,
    /// Agender pride flag (black, grey, white, green)
    Agender,
    /// Polyamory pride flag (blue, red, black with pi symbol)
    Polyamory,
    /// Bear pride flag (browns) - original default
    Bear,
    /// Progress pride flag (chevron with multiple identities)
    /// Note: This renders as horizontal stripes; proper chevron rendering would require additional logic
    Progress,
}

impl PrideFlagPreset {
    /// Returns the traditional color palette for this pride flag
    ///
    /// Colors are returned as u32 hex RGB values (0xRRGGBB)
    pub fn colors(self) -> Vec<u32> {
        match self {
            PrideFlagPreset::Rainbow => vec![
                0xE40303, // Red
                0xFF8C00, // Orange
                0xFFED00, // Yellow
                0x008026, // Green
                0x24408E, // Indigo
                0x732982, // Violet
            ],
            PrideFlagPreset::RainbowGilbertBaker => vec![
                0xFF69B4, // Hot pink
                0xFF0000, // Red
                0xFF8C00, // Orange
                0xFFFF00, // Yellow
                0x008000, // Green
                0x00FFFF, // Turquoise
                0x0000FF, // Indigo
                0x8B00FF, // Violet
            ],
            PrideFlagPreset::Trans => vec![
                0x5BCEFA, // Light blue
                0xF5A9B8, // Pink
                0xFFFFFF, // White
                0xF5A9B8, // Pink
                0x5BCEFA, // Light blue
            ],
            PrideFlagPreset::Bisexual => vec![
                0xD60270, // Pink
                0xD60270, // Pink
                0x9B4F96, // Purple
                0x0038A8, // Blue
                0x0038A8, // Blue
            ],
            PrideFlagPreset::Pansexual => vec![
                0xFF218C, // Pink
                0xFFD800, // Yellow
                0x21B1FF, // Blue
            ],
            PrideFlagPreset::Lesbian => vec![
                0xD62800, // Dark orange
                0xFF9B56, // Orange
                0xFFFFFF, // White
                0xD462A6, // Pink
                0xA40062, // Dark pink
            ],
            PrideFlagPreset::Asexual => vec![
                0x000000, // Black
                0xA3A3A3, // Grey
                0xFFFFFF, // White
                0x800080, // Purple
            ],
            PrideFlagPreset::Nonbinary => vec![
                0xFCF434, // Yellow
                0xFFFFFF, // White
                0x9C59D1, // Purple
                0x2C2C2C, // Black
            ],
            PrideFlagPreset::Genderqueer => vec![
                0xB57EDC, // Lavender
                0xFFFFFF, // White
                0x4A8123, // Green
            ],
            PrideFlagPreset::Genderfluid => vec![
                0xFF75A2, // Pink
                0xFFFFFF, // White
                0xC011D7, // Purple
                0x000000, // Black
                0x333EBD, // Blue
            ],
            PrideFlagPreset::Aromantic => vec![
                0x3DA542, // Green
                0xA7D379, // Light green
                0xFFFFFF, // White
                0xA9A9A9, // Grey
                0x000000, // Black
            ],
            PrideFlagPreset::Agender => vec![
                0x000000, // Black
                0xB9B9B9, // Grey
                0xFFFFFF, // White
                0xB8F483, // Light green
                0xFFFFFF, // White
                0xB9B9B9, // Grey
                0x000000, // Black
            ],
            PrideFlagPreset::Polyamory => vec![
                0x0000FF, // Blue
                0xFF0000, // Red
                0x000000, // Black
            ],
            PrideFlagPreset::Bear => vec![
                0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41,
                0x89491D, 0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
            ],
            PrideFlagPreset::Progress => vec![
                0xFFFFFF, // White
                0xF5A9B8, // Pink
                0x5BCEFA, // Light blue
                0x613915, // Brown
                0x000000, // Black
                0xE40303, // Red
                0xFF8C00, // Orange
                0xFFED00, // Yellow
                0x008026, // Green
                0x24408E, // Indigo
                0x732982, // Violet
            ],
        }
    }

    /// Returns a human-readable name for this pride flag
    pub fn display_name(self) -> &'static str {
        match self {
            PrideFlagPreset::Rainbow => "Rainbow Pride (6-stripe)",
            PrideFlagPreset::RainbowGilbertBaker => "Gilbert Baker Rainbow (8-stripe)",
            PrideFlagPreset::Trans => "Transgender Pride",
            PrideFlagPreset::Bisexual => "Bisexual Pride",
            PrideFlagPreset::Pansexual => "Pansexual Pride",
            PrideFlagPreset::Lesbian => "Lesbian Pride",
            PrideFlagPreset::Asexual => "Asexual Pride",
            PrideFlagPreset::Nonbinary => "Non-binary Pride",
            PrideFlagPreset::Genderqueer => "Genderqueer Pride",
            PrideFlagPreset::Genderfluid => "Genderfluid Pride",
            PrideFlagPreset::Aromantic => "Aromantic Pride",
            PrideFlagPreset::Agender => "Agender Pride",
            PrideFlagPreset::Polyamory => "Polyamory Pride",
            PrideFlagPreset::Bear => "Bear Pride",
            PrideFlagPreset::Progress => "Progress Pride",
        }
    }

    /// Returns whether this flag traditionally has a bear paw overlay
    pub fn has_bear_paw(self) -> bool {
        matches!(self, PrideFlagPreset::Bear)
    }
}

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
    /// Color palette for the flag (RGB hex values)
    pub colors: Vec<u32>,
    /// Whether to include bear paw overlay
    pub include_paw: bool,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
    /// Whether to use transparent background (only for formats that support it)
    pub transparent: bool,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_format: OutputFormat::Png,
            colors: PrideFlagPreset::Rainbow.colors(),
            include_paw: false,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        }
    }
}

impl FlagConfig {
    /// Creates a configuration from a device preset
    ///
    /// Uses sensible defaults for paw sizing and positioning
    #[allow(dead_code)]
    pub fn from_preset(preset: DevicePreset) -> Self {
        let (width, height) = preset.into();
        Self {
            width,
            height,
            output_format: OutputFormat::Png,
            colors: PrideFlagPreset::Rainbow.colors(),
            include_paw: false,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        }
    }

    /// Creates a configuration from a pride flag preset
    ///
    /// Uses the traditional colors for the specified pride flag
    #[allow(dead_code)]
    pub fn from_pride_preset(flag: PrideFlagPreset, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            output_format: OutputFormat::Png,
            colors: flag.colors(),
            include_paw: flag.has_bear_paw(),
            paw_size_ratio: 0.35,
            center_paw: true,
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
        if self.colors.len() > 100 {
            return Err(FlagError::InvalidConfig(
                "Maximum 100 color stripes allowed".to_string(),
            ));
        }
        if self.include_paw && !(0.01..=1.0).contains(&self.paw_size_ratio) {
            return Err(FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Query parameters for flag generation endpoint
#[derive(Debug, Deserialize)]
pub struct FlagQuery {
    /// Pride flag preset (rainbow, trans, bi, etc.)
    pub flag: Option<PrideFlagPreset>,
    /// Device preset for standard dimensions
    pub preset: Option<DevicePreset>,
    /// Custom width in pixels (overrides preset)
    pub width: Option<u32>,
    /// Custom height in pixels (overrides preset)
    pub height: Option<u32>,
    /// Custom colors as comma-separated hex values (e.g., "FF0000,00FF00,0000FF")
    /// Overrides flag preset colors if provided
    pub colors: Option<String>,
    /// Output format (png, jpeg, webp)
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// Whether to include bear paw overlay (default: auto-detected from flag type)
    pub include_paw: Option<bool>,
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
            include_paw: true,
            paw_size_ratio: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
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
            colors: vec![0xFF0000; 101],
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
    fn test_pride_flag_preset_rainbow() {
        let colors = PrideFlagPreset::Rainbow.colors();
        assert_eq!(colors.len(), 6);
        assert_eq!(colors[0], 0xE40303); // Red
    }

    #[test]
    fn test_pride_flag_preset_trans() {
        let colors = PrideFlagPreset::Trans.colors();
        assert_eq!(colors.len(), 5);
        assert!(colors.contains(&0x5BCEFA)); // Light blue
        assert!(colors.contains(&0xFFFFFF)); // White
    }

    #[test]
    fn test_pride_flag_preset_bear_has_paw() {
        assert!(PrideFlagPreset::Bear.has_bear_paw());
        assert!(!PrideFlagPreset::Rainbow.has_bear_paw());
        assert!(!PrideFlagPreset::Trans.has_bear_paw());
    }

    #[test]
    fn test_flag_config_from_pride_preset() {
        let config = FlagConfig::from_pride_preset(PrideFlagPreset::Trans, 1920, 1080);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.colors.len(), 5);
        assert!(!config.include_paw);
    }

    #[test]
    fn test_flag_config_from_bear_preset() {
        let config = FlagConfig::from_pride_preset(PrideFlagPreset::Bear, 1920, 1080);
        assert!(config.include_paw);
        assert!(config.colors.len() > 5);
    }

    #[test]
    fn test_output_format_mime_types() {
        assert_eq!(OutputFormat::Png.mime_type(), "image/png");
        assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(OutputFormat::WebP.mime_type(), "image/webp");
    }
}
