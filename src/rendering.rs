//! Image rendering utilities for the Bear Flag API.

use crate::{
    config::{FlagConfig, OutputFormat},
    error::FlagError,
};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform};

/// Embeds `assets/bear_paw.svg` directly into the binary.
const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

/// Traditional bear pride palette: warm browns transitioning to deep browns/blacks.
const BEAR_PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

/// Number of pixels over which adjacent color stripes smoothly blend.
const SMOOTH_WIDTH: u32 = 16;

/// Renders SVG data to an RGBA image buffer at the specified size.
///
/// The SVG is scaled proportionally so its largest dimension matches `target_size`.
///
/// # Errors
///
/// Returns `FlagError::SvgParse` if the SVG cannot be parsed or rendered.
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

/// Linear interpolation between two u8 channel values.
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t).mul_add(a as f32, t * b as f32)).round() as u8
}

/// Draws the bear pride flag with smooth color transitions.
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

            // Smooth blending in the last SMOOTH_WIDTH pixels if not the last stripe.
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

/// Composites the source image onto the destination using proper alpha blending.
fn composite_with_alpha(dst: &mut RgbaImage, src: &RgbaImage, offset_x: u32, offset_y: u32) {
    for (src_x, src_y, src_pixel) in src.enumerate_pixels() {
        let dst_x = offset_x + src_x;
        let dst_y = offset_y + src_y;

        // Skip pixels outside destination bounds.
        if dst_x >= dst.width() || dst_y >= dst.height() {
            continue;
        }

        let src_alpha = src_pixel[3] as f32 / 255.0;

        // Skip fully transparent pixels for performance.
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

/// Generates the complete gay bear pride flag and returns image bytes.
///
/// Creates a flag with smooth color transitions and a bear paw overlay,
/// returning the encoded image data.
///
/// # Errors
///
/// Returns errors if SVG rendering fails, image buffer creation fails,
/// or the image cannot be encoded.
pub fn generate_flag_bytes(config: &FlagConfig) -> Result<Vec<u8>, FlagError> {
    config.validate()?;

    let mut img = if config.transparent {
        // Initialize with transparent background.
        RgbaImage::from_pixel(config.width, config.height, Rgba([0, 0, 0, 0]))
    } else {
        // Initialize with opaque background.
        RgbaImage::new(config.width, config.height)
    };

    let stripe_width = config.width / BEAR_PALETTE.len() as u32;
    draw_bear_stripes(&mut img, &BEAR_PALETTE, stripe_width, config.height);

    let paw_size = (config.height as f32 * config.paw_size_ratio) as u32;
    let bear_paw = render_svg_to_rgba(BEAR_PAW_SVG, paw_size)?;

    let (paw_x, paw_y) = if config.center_paw {
        // Center the paw in the flag.
        let x = (config.width.saturating_sub(bear_paw.width())) / 2;
        let y = (config.height.saturating_sub(bear_paw.height())) / 2;
        (x, y)
    } else {
        // Bottom-left positioning (classic).
        let x = 0;
        let y = config.height.saturating_sub(bear_paw.height());
        (x, y)
    };

    composite_with_alpha(&mut img, &bear_paw, paw_x, paw_y);

    // Encode to bytes.
    let format: ImageFormat = config.output_format.into();
    let mut bytes = Vec::new();

    // JPEG doesn't support alpha channel, convert RGBA to RGB.
    if matches!(config.output_format, OutputFormat::Jpeg) {
        let rgb_img = DynamicImage::ImageRgba8(img).to_rgb8();
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
    fn lerp_u8_respects_endpoints() {
        assert_eq!(lerp_u8(0, 255, 0.0), 0);
        assert_eq!(lerp_u8(0, 255, 1.0), 255);
    }

    #[test]
    fn lerp_u8_handles_midpoint() {
        let result = lerp_u8(0, 100, 0.5);
        assert!(
            (result as i32 - 50).abs() <= 1,
            "Expected ~50, got {result}"
        );
    }

    #[test]
    fn render_svg_produces_image() {
        let result = render_svg_to_rgba(BEAR_PAW_SVG, 100);
        assert!(result.is_ok(), "SVG rendering failed: {:?}", result.err());

        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn composite_respects_transparency() {
        let mut dst = RgbaImage::new(10, 10);
        for pixel in dst.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        let mut src = RgbaImage::new(5, 5);
        for pixel in src.pixels_mut() {
            *pixel = Rgba([0, 0, 255, 128]);
        }

        composite_with_alpha(&mut dst, &src, 0, 0);

        let blended_pixel = dst.get_pixel(0, 0);
        assert!(blended_pixel[0] > 0, "Red channel should have contribution");
        assert!(
            blended_pixel[2] > 0,
            "Blue channel should have contribution"
        );
    }

    #[test]
    fn generate_flag_bytes_produces_image() {
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
    fn generate_flag_bytes_handles_jpeg() {
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
