//! Configuration types for flag generation

use crate::format::OutputFormat;

/// Preset device configurations with appropriate dimensions for wallpapers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    IPhone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    IPhone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    IPhone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    IPhoneSE,
    /// Desktop 4K - 3840 x 2160
    Desktop4K,
    /// Desktop 1440p - 2560 x 1440
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    Desktop1080p,
}

impl DevicePreset {
    /// Returns a human-readable name for the device preset
    pub fn display_name(self) -> &'static str {
        match self {
            DevicePreset::IPhone14ProMax => "iPhone 14/13/12 Pro Max",
            DevicePreset::IPhone14Pro => "iPhone 14/13/12 Pro",
            DevicePreset::IPhone14 => "iPhone 14/13/12",
            DevicePreset::IPhoneSE => "iPhone SE (3rd gen)",
            DevicePreset::Desktop4K => "Desktop 4K",
            DevicePreset::Desktop1440p => "Desktop 1440p",
            DevicePreset::Desktop1080p => "Desktop 1080p",
        }
    }

    /// Returns all available device presets
    pub fn all() -> &'static [DevicePreset] {
        &[
            DevicePreset::IPhone14ProMax,
            DevicePreset::IPhone14Pro,
            DevicePreset::IPhone14,
            DevicePreset::IPhoneSE,
            DevicePreset::Desktop4K,
            DevicePreset::Desktop1440p,
            DevicePreset::Desktop1080p,
        ]
    }
}

impl From<DevicePreset> for (u32, u32) {
    /// Converts a device preset to (width, height) dimensions
    fn from(preset: DevicePreset) -> Self {
        match preset {
            DevicePreset::IPhone14ProMax => (2796, 1290),
            DevicePreset::IPhone14Pro => (2556, 1179),
            DevicePreset::IPhone14 => (2532, 1170),
            DevicePreset::IPhoneSE => (1334, 750),
            DevicePreset::Desktop4K => (3840, 2160),
            DevicePreset::Desktop1440p => (2560, 1440),
            DevicePreset::Desktop1080p => (1920, 1080),
        }
    }
}

/// Configuration for flag generation
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Output image width in pixels
    pub width: u32,
    /// Output image height in pixels
    pub height: u32,
    /// Path where the flag image will be saved
    pub output_path: String,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
    /// Output image format
    pub output_format: OutputFormat,
    /// Quality setting for lossy formats (0-100)
    pub quality: u8,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_path: "bear_flag.png".to_string(),
            paw_size_ratio: 0.35,
            center_paw: true,
            output_format: OutputFormat::Png,
            quality: 95,
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
            output_path: format!("bear_flag_{}x{}.png", width, height),
            paw_size_ratio: 0.35,
            center_paw: true,
            output_format: OutputFormat::Png,
            quality: 95,
        }
    }

    /// Ensures the output path has the correct extension for the selected format
    pub fn ensure_extension(&mut self) {
        let expected_ext = self.output_format.extension();
        let path = std::path::Path::new(&self.output_path);
        
        if path.extension().and_then(|e| e.to_str()) != Some(expected_ext) {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                self.output_path = format!("{}.{}", stem, expected_ext);
            } else {
                self.output_path = format!("bear_flag.{}", expected_ext);
            }
        }
    }

    /// Validates the configuration parameters
    ///
    /// # Errors
    ///
    /// Returns `FlagError::InvalidConfig` if any parameters are invalid
    pub fn validate(&self) -> Result<(), crate::error::FlagError> {
        if self.width == 0 || self.height == 0 {
            return Err(crate::error::FlagError::InvalidConfig(
                "Width and height must be non-zero".to_string(),
            ));
        }
        if !(0.01..=1.0).contains(&self.paw_size_ratio) {
            return Err(crate::error::FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }
        if self.quality > 100 {
            return Err(crate::error::FlagError::InvalidConfig(
                "Quality must be between 0 and 100".to_string(),
            ));
        }
        Ok(())
    }
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
    fn test_device_preset_desktop_1080p() {
        let (width, height) = DevicePreset::Desktop1080p.into();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_flag_config_from_preset() {
        let config = FlagConfig::from_preset(DevicePreset::IPhone14ProMax);
        assert_eq!(config.width, 2796);
        assert_eq!(config.height, 1290);
        assert_eq!(config.paw_size_ratio, 0.35);
        assert!(config.center_paw);
        assert_eq!(config.output_path, "bear_flag_2796x1290.png");
    }

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
    fn test_ensure_extension() {
        let mut config = FlagConfig {
            output_path: "test.txt".to_string(),
            output_format: OutputFormat::Png,
            ..Default::default()
        };
        config.ensure_extension();
        assert_eq!(config.output_path, "test.png");
    }

    #[test]
    fn test_device_preset_display_names() {
        assert_eq!(
            DevicePreset::IPhone14ProMax.display_name(),
            "iPhone 14/13/12 Pro Max"
        );
        assert_eq!(
            DevicePreset::IPhone14Pro.display_name(),
            "iPhone 14/13/12 Pro"
        );
        assert_eq!(DevicePreset::IPhone14.display_name(), "iPhone 14/13/12");
        assert_eq!(DevicePreset::IPhoneSE.display_name(), "iPhone SE (3rd gen)");
        assert_eq!(DevicePreset::Desktop4K.display_name(), "Desktop 4K");
    }
}
