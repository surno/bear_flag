//! Gay Bear Flag Generator
//!
//! Generates a high-quality gay bear pride flag with smooth color gradients
//! and a centered bear paw overlay. The flag combines the traditional bear
//! pride colors with proper alpha compositing for professional results.

use clap::{Parser, ValueEnum};
use image::{ImageBuffer, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::usvg::Transform;
use thiserror::Error;

/// Embeds assets/bear_paw.svg directly into the binary
const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

/// Traditional bear pride palette: warm browns transitioning to deep browns/blacks
/// Colors chosen to represent the bear community's diversity and warmth
const BEAR_PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

/// Number of pixels over which adjacent color stripes smoothly blend
const SMOOTH_WIDTH: u32 = 16;

/// Preset device configurations with appropriate dimensions for wallpapers
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    #[value(name = "iphone-14-pro-max")]
    IPhone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    #[value(name = "iphone-14-pro")]
    IPhone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    #[value(name = "iphone-14")]
    IPhone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    #[value(name = "iphone-se")]
    IPhoneSE,
    /// Desktop 4K - 3840 x 2160
    #[value(name = "desktop-4k")]
    Desktop4K,
    /// Desktop 1440p - 2560 x 1440
    #[value(name = "desktop-1440p")]
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    #[value(name = "desktop-1080p")]
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
            DevicePreset::Desktop4K => "Desktop 4K",
            DevicePreset::Desktop1440p => "Desktop 1440p",
            DevicePreset::Desktop1080p => "Desktop 1080p",
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

    #[error("Failed to save image to {path}: {source}")]
    ImageSave {
        path: String,
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
    /// Path where the flag image will be saved
    pub output_path: String,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_path: "bear_flag.png".to_string(),
            paw_size_ratio: 0.35,
            center_paw: true,
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
        }
    }
}

impl FlagConfig {
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
        if !(0.01..=1.0).contains(&self.paw_size_ratio) {
            return Err(FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Renders SVG data to an RGBA image buffer at the specified size
///
/// The SVG is scaled proportionally so its largest dimension matches `target_size`.
///
/// # Arguments
///
/// * `svg_data` - Raw SVG file data
/// * `target_size` - Target size for the largest dimension (width or height)
///
/// # Errors
///
/// Returns `FlagError::SvgParse` if the SVG cannot be parsed or rendered
fn render_svg_to_rgba(svg_data: &[u8], target_size: u32) -> Result<RgbaImage, FlagError> {
    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
        .map_err(|e| FlagError::SvgParse(e.to_string()))?;

    let svg_size = tree.size();
    let max_dim = svg_size.width().max(svg_size.height());
    let scale = target_size as f32 / max_dim;

    let width_px = (svg_size.width() * scale).ceil() as u32;
    let height_px = (svg_size.height() * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width_px, height_px).ok_or_else(|| FlagError::BufferCreation {
        width: width_px,
        height: height_px,
    })?;

    let transform = Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec()).ok_or_else(
        || FlagError::BufferCreation {
            width: pixmap.width(),
            height: pixmap.height(),
        },
    )
}

/// Linear interpolation between two u8 channel values
///
/// # Arguments
///
/// * `a` - Start value
/// * `b` - End value
/// * `t` - Interpolation factor (0.0 = a, 1.0 = b)
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t).mul_add(a as f32, t * b as f32)).round() as u8
}

/// Draws the bear pride flag with smooth color transitions
///
/// Creates horizontal stripes from the given palette with smooth gradients
/// between adjacent colors for a professional appearance.
///
/// # Arguments
///
/// * `img` - Target image buffer to draw into
/// * `palette` - Array of RGB colors as u32 hex values (0xRRGGBB)
/// * `stripe_width` - Width of each color stripe in pixels
/// * `height` - Height of the flag in pixels
fn draw_bear_stripes(img: &mut RgbaImage, palette: &[u32], stripe_width: u32, height: u32) {
    for (i, &hex) in palette.iter().enumerate() {
        let next_hex = palette.get(i + 1).copied().unwrap_or(hex);

        let rgb_cur = [
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        ];
        let rgb_next = [
            ((next_hex >> 16) & 0xFF) as u8,
            ((next_hex >> 8) & 0xFF) as u8,
            (next_hex & 0xFF) as u8,
        ];

        let x_start = i as u32 * stripe_width;
        let x_end = ((i + 1) as u32 * stripe_width).min(img.width());

        for x in x_start..x_end {
            let dist_from_end = x_end.saturating_sub(x + 1);

            // Smooth blending in the last SMOOTH_WIDTH pixels if not the last stripe
            let blend_factor = if dist_from_end < SMOOTH_WIDTH && i + 1 < palette.len() {
                1.0 - (dist_from_end as f32 / SMOOTH_WIDTH as f32)
            } else {
                0.0
            };

            let blended = Rgba([
                lerp_u8(rgb_cur[0], rgb_next[0], blend_factor),
                lerp_u8(rgb_cur[1], rgb_next[1], blend_factor),
                lerp_u8(rgb_cur[2], rgb_next[2], blend_factor),
                255,
            ]);

            for y in 0..height {
                img.put_pixel(x, y, blended);
            }
        }
    }
}

/// Composites the source image onto the destination using proper alpha blending
///
/// Uses "over" compositing: `result = src_alpha * src + (1 - src_alpha) * dst`
///
/// # Arguments
///
/// * `dst` - Destination image (background)
/// * `src` - Source image to composite (foreground)
/// * `offset_x` - Horizontal offset for source placement
/// * `offset_y` - Vertical offset for source placement
fn composite_with_alpha(dst: &mut RgbaImage, src: &RgbaImage, offset_x: u32, offset_y: u32) {
    for (src_x, src_y, src_pixel) in src.enumerate_pixels() {
        let dst_x = offset_x + src_x;
        let dst_y = offset_y + src_y;

        // Skip pixels outside destination bounds
        if dst_x >= dst.width() || dst_y >= dst.height() {
            continue;
        }

        let src_alpha = src_pixel[3] as f32 / 255.0;

        // Skip fully transparent pixels for performance
        if src_alpha < 0.001 {
            continue;
        }

        let dst_pixel = dst.get_pixel(dst_x, dst_y);
        let inv_alpha = 1.0 - src_alpha;

        let blended = Rgba([
            (src_alpha.mul_add(src_pixel[0] as f32, inv_alpha * dst_pixel[0] as f32)).round() as u8,
            (src_alpha.mul_add(src_pixel[1] as f32, inv_alpha * dst_pixel[1] as f32)).round() as u8,
            (src_alpha.mul_add(src_pixel[2] as f32, inv_alpha * dst_pixel[2] as f32)).round() as u8,
            255,
        ]);

        dst.put_pixel(dst_x, dst_y, blended);
    }
}

/// Generates the complete gay bear pride flag
///
/// Creates a flag with smooth color transitions and a bear paw overlay,
/// saving the result to the configured output path.
///
/// # Arguments
///
/// * `config` - Configuration specifying dimensions, output path, and styling
///
/// # Errors
///
/// Returns errors if SVG rendering fails, image buffer creation fails,
/// or the image cannot be saved to the output path.
pub fn generate_flag(config: &FlagConfig) -> Result<(), FlagError> {
    config.validate()?;

    let mut img = RgbaImage::new(config.width, config.height);

    let stripe_width = config.width / BEAR_PALETTE.len() as u32;
    draw_bear_stripes(&mut img, &BEAR_PALETTE, stripe_width, config.height);

    let paw_size = (config.height as f32 * config.paw_size_ratio) as u32;
    let bear_paw = render_svg_to_rgba(BEAR_PAW_SVG, paw_size)?;

    let (paw_x, paw_y) = if config.center_paw {
        // Center the paw in the flag
        let x = (config.width.saturating_sub(bear_paw.width())) / 2;
        let y = (config.height.saturating_sub(bear_paw.height())) / 2;
        (x, y)
    } else {
        // Bottom-left positioning (classic)
        let x = 0;
        let y = config.height.saturating_sub(bear_paw.height());
        (x, y)
    };

    composite_with_alpha(&mut img, &bear_paw, paw_x, paw_y);

    img.save(&config.output_path)
        .map_err(|e| FlagError::ImageSave {
            path: config.output_path.clone(),
            source: e,
        })?;

    Ok(())
}

/// Command-line arguments for the bear flag generator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Device preset to generate wallpaper for
    #[arg(short, long, value_enum, default_value = "desktop-4k")]
    device: DevicePreset,

    /// Custom output path (overrides default based on dimensions)
    #[arg(short, long)]
    output: Option<String>,

    /// Custom width in pixels (overrides device preset)
    #[arg(long)]
    width: Option<u32>,

    /// Custom height in pixels (overrides device preset)
    #[arg(long)]
    height: Option<u32>,

    /// Size of the bear paw as a fraction of flag height (0.01-1.0)
    #[arg(long, default_value = "0.35")]
    paw_size: f32,

    /// Place paw in bottom-left instead of center
    #[arg(long)]
    bottom_left: bool,
}

fn main() -> Result<(), FlagError> {
    let cli = Cli::parse();

    let mut config = if let (Some(width), Some(height)) = (cli.width, cli.height) {
        // Custom dimensions override device preset
        FlagConfig {
            width,
            height,
            output_path: format!("bear_flag_{}x{}.png", width, height),
            paw_size_ratio: cli.paw_size,
            center_paw: !cli.bottom_left,
        }
    } else {
        // Use device preset
        let mut cfg = FlagConfig::from_preset(cli.device);
        cfg.paw_size_ratio = cli.paw_size;
        cfg.center_paw = !cli.bottom_left;
        cfg
    };

    // Apply custom output path if provided
    if let Some(output) = cli.output {
        config.output_path = output;
    }

    let device_name = if cli.width.is_some() && cli.height.is_some() {
        "Custom".to_string()
    } else {
        cli.device.display_name().to_string()
    };

    println!("Generating gay bear pride flag...");
    println!("  Device: {}", device_name);
    println!("  Dimensions: {}x{}", config.width, config.height);
    println!("  Output: {}", config.output_path);
    println!(
        "  Paw position: {}",
        if config.center_paw {
            "centered"
        } else {
            "bottom-left"
        }
    );

    generate_flag(&config)?;

    println!("? Flag generated successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_u8_endpoints() {
        assert_eq!(lerp_u8(0, 255, 0.0), 0);
        assert_eq!(lerp_u8(0, 255, 1.0), 255);
    }

    #[test]
    fn test_lerp_u8_midpoint() {
        let result = lerp_u8(0, 100, 0.5);
        assert!(
            (result as i32 - 50).abs() <= 1,
            "Expected ~50, got {}",
            result
        );
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
    fn test_generate_flag_small() {
        let config = FlagConfig {
            width: 140,
            height: 80,
            output_path: "test_flag_small.png".to_string(),
            paw_size_ratio: 0.3,
            center_paw: true,
        };

        let result = generate_flag(&config);
        assert!(result.is_ok(), "Flag generation failed: {:?}", result.err());

        // Cleanup
        let _ = std::fs::remove_file(&config.output_path);
    }

    #[test]
    fn test_render_svg_to_rgba() {
        let result = render_svg_to_rgba(BEAR_PAW_SVG, 100);
        assert!(result.is_ok(), "SVG rendering failed: {:?}", result.err());

        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn test_composite_respects_transparency() {
        let mut dst = RgbaImage::new(10, 10);
        // Fill with red
        for pixel in dst.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        let mut src = RgbaImage::new(5, 5);
        // Fill with semi-transparent blue
        for pixel in src.pixels_mut() {
            *pixel = Rgba([0, 0, 255, 128]);
        }

        composite_with_alpha(&mut dst, &src, 0, 0);

        let blended_pixel = dst.get_pixel(0, 0);
        // Should be a purple-ish blend
        assert!(blended_pixel[0] > 0, "Red channel should have contribution");
        assert!(
            blended_pixel[2] > 0,
            "Blue channel should have contribution"
        );
    }

    #[test]
    fn test_device_preset_iphone_14_pro_max() {
        let (width, height) = DevicePreset::IPhone14ProMax.into();
        assert_eq!(width, 2796);
        assert_eq!(height, 1290);
    }

    #[test]
    fn test_device_preset_iphone_14_pro() {
        let (width, height) = DevicePreset::IPhone14Pro.into();
        assert_eq!(width, 2556);
        assert_eq!(height, 1179);
    }

    #[test]
    fn test_device_preset_iphone_14() {
        let (width, height) = DevicePreset::IPhone14.into();
        assert_eq!(width, 2532);
        assert_eq!(height, 1170);
    }

    #[test]
    fn test_device_preset_iphone_se() {
        let (width, height) = DevicePreset::IPhoneSE.into();
        assert_eq!(width, 1334);
        assert_eq!(height, 750);
    }

    #[test]
    fn test_device_preset_desktop_4k() {
        let (width, height) = DevicePreset::Desktop4K.into();
        assert_eq!(width, 3840);
        assert_eq!(height, 2160);
    }

    #[test]
    fn test_device_preset_desktop_1440p() {
        let (width, height) = DevicePreset::Desktop1440p.into();
        assert_eq!(width, 2560);
        assert_eq!(height, 1440);
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
    fn test_flag_config_from_preset_desktop_1080p() {
        let config = FlagConfig::from_preset(DevicePreset::Desktop1080p);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.output_path, "bear_flag_1920x1080.png");
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
        assert_eq!(DevicePreset::Desktop1440p.display_name(), "Desktop 1440p");
        assert_eq!(DevicePreset::Desktop1080p.display_name(), "Desktop 1080p");
    }

    #[test]
    fn test_generate_flag_iphone_preset() {
        let config = FlagConfig {
            width: 1334,
            height: 750,
            output_path: "test_flag_iphone.png".to_string(),
            paw_size_ratio: 0.3,
            center_paw: true,
        };

        let result = generate_flag(&config);
        assert!(
            result.is_ok(),
            "iPhone flag generation failed: {:?}",
            result.err()
        );

        // Cleanup
        let _ = std::fs::remove_file(&config.output_path);
    }
}
