//! Data models for device presets, output formats, and query parameters

use image::ImageFormat;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_output_format_mime_types() {
        assert_eq!(OutputFormat::Png.mime_type(), "image/png");
        assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(OutputFormat::WebP.mime_type(), "image/webp");
    }
}
