//! Core flag generation logic

use crate::constants::{BEAR_PALETTE, BEAR_PAW_SVG};
use crate::rendering::{composite_with_alpha, draw_bear_stripes, render_svg_to_rgba};
use crate::types::{FlagConfig, FlagError, OutputFormat};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

/// Generates the complete gay bear pride flag and returns image bytes
///
/// Creates a flag with smooth color transitions and a bear paw overlay,
/// returning the encoded image data.
///
/// # Arguments
///
/// * `config` - Configuration specifying dimensions, output format, and styling
///
/// # Errors
///
/// Returns errors if SVG rendering fails, image buffer creation fails,
/// or the image cannot be encoded.
pub fn generate_flag_bytes(config: &FlagConfig) -> Result<Vec<u8>, FlagError> {
    config.validate()?;

    let mut img = if config.transparent {
        // Initialize with transparent background
        RgbaImage::from_pixel(config.width, config.height, Rgba([0, 0, 0, 0]))
    } else {
        // Initialize with opaque background
        RgbaImage::new(config.width, config.height)
    };

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

    // Encode to bytes
    let format: ImageFormat = config.output_format.into();
    let mut bytes = Vec::new();

    // JPEG doesn't support alpha channel, convert RGBA to RGB
    if matches!(config.output_format, OutputFormat::Jpeg) {
        let dynamic = DynamicImage::ImageRgba8(img);
        let rgb_img = dynamic.to_rgb8();
        rgb_img
            .write_to(&mut std::io::Cursor::new(&mut bytes), format)
            .map_err(|e| FlagError::ImageEncode {
                format: config.output_format,
                source: e,
            })?;
    } else {
        img.write_to(&mut std::io::Cursor::new(&mut bytes), format)
            .map_err(|e| FlagError::ImageEncode {
                format: config.output_format,
                source: e,
            })?;
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_flag_bytes_small() {
        let config = FlagConfig {
            width: 140,
            height: 80,
            output_format: OutputFormat::Png,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: false,
        };

        let result = generate_flag_bytes(&config);
        assert!(result.is_ok(), "Flag generation failed: {:?}", result.err());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "Generated image should not be empty");
    }

    #[test]
    fn test_generate_flag_bytes_jpeg() {
        let config = FlagConfig {
            width: 320,
            height: 240,
            output_format: OutputFormat::Jpeg,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: false,
        };

        let result = generate_flag_bytes(&config);
        assert!(
            result.is_ok(),
            "JPEG flag generation failed: {:?}",
            result.err()
        );

        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "Generated JPEG should not be empty");
    }
}
