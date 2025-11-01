use image::{ImageBuffer, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::usvg::Transform;
use std::io::Error;
use std::io::ErrorKind;

// Embeds assets/bear_paw.svg directly into the binary
const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

const PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

const SMOOTH_WIDTH: u32 = 16; // pixels over which adjacent bars blend

fn get_svg_as_rgba(svg_data: &[u8], target_size: u32) -> Result<RgbaImage, Error> {
    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to parse SVG"))?;

    // --- scale SVG proportionally so its largest side == target_size ---
    let svg_size = tree.size();
    let max_dim = svg_size.width().max(svg_size.height());
    let scale = target_size as f32 / max_dim;

    let width_px = (svg_size.width() * scale).ceil() as u32;
    let height_px = (svg_size.height() * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width_px, height_px)
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create pixmap"))?;

    let transform = Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let svg_image = ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create image buffer"))?;

    Ok(svg_image)
}

// Linear‑interpolate two u8 channel values
fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t) * a as f32 + t * b as f32).round() as u8
}

/// Draw a drop shadow around the paw print for better visibility
fn draw_paw_with_shadow(
    img: &mut RgbaImage,
    paw: &RgbaImage,
    center_x: i32,
    center_y: i32,
    shadow_offset: i32,
    shadow_blur: i32,
) {
    let img_w = img.width() as i32;
    let img_h = img.height() as i32;

    // Draw shadow first (darker, blurred)
    for (px, py, pixel) in paw.enumerate_pixels() {
        if pixel[3] != 0 {
            let px_i = px as i32;
            let py_i = py as i32;

            // Draw shadow with blur
            for dy in -shadow_blur..=shadow_blur {
                for dx in -shadow_blur..=shadow_blur {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= shadow_blur * shadow_blur {
                        let shadow_x = center_x + px_i + shadow_offset + dx;
                        let shadow_y = center_y + py_i + shadow_offset + dy;

                        if shadow_x >= 0 && shadow_x < img_w && shadow_y >= 0 && shadow_y < img_h {
                            let alpha = ((pixel[3] as f32)
                                * (1.0 - dist_sq as f32 / (shadow_blur * shadow_blur) as f32)
                                * 0.3)
                                .min(255.0) as u8;
                            if alpha > 0 {
                                let existing = img.get_pixel(shadow_x as u32, shadow_y as u32);
                                let shadow = Rgba([0, 0, 0, alpha]);
                                let blended = Rgba([
                                    (existing[0] as u16 * (255 - alpha) as u16 / 255
                                        + shadow[0] as u16 * alpha as u16 / 255)
                                        as u8,
                                    (existing[1] as u16 * (255 - alpha) as u16 / 255
                                        + shadow[1] as u16 * alpha as u16 / 255)
                                        as u8,
                                    (existing[2] as u16 * (255 - alpha) as u16 / 255
                                        + shadow[2] as u16 * alpha as u16 / 255)
                                        as u8,
                                    255,
                                ]);
                                img.put_pixel(shadow_x as u32, shadow_y as u32, blended);
                            }
                        }
                    }
                }
            }
        }
    }

    // Draw the paw print itself
    for (px, py, pixel) in paw.enumerate_pixels() {
        if pixel[3] != 0 {
            let x = center_x + px as i32;
            let y = center_y + py as i32;

            if x >= 0 && x < img_w && y >= 0 && y < img_h {
                let existing = img.get_pixel(x as u32, y as u32);
                let alpha = pixel[3] as f32 / 255.0;
                let blended = Rgba([
                    ((existing[0] as f32 * (1.0 - alpha) + pixel[0] as f32 * alpha).round()) as u8,
                    ((existing[1] as f32 * (1.0 - alpha) + pixel[1] as f32 * alpha).round()) as u8,
                    ((existing[2] as f32 * (1.0 - alpha) + pixel[2] as f32 * alpha).round()) as u8,
                    255,
                ]);
                img.put_pixel(x as u32, y as u32, blended);
            }
        }
    }
}

fn draw_bear_flag(img: &mut RgbaImage, palette: &[u32], stripe: u32, h: u32) {
    for (i, &hex) in palette.iter().enumerate() {
        let next_hex = if i + 1 < palette.len() {
            palette[i + 1]
        } else {
            hex
        };

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

        let x_start = i as u32 * stripe;
        let x_end = (i as u32 + 1) * stripe; // exclusive RHS

        for x in x_start..x_end {
            // distance from the right boundary of this bar
            let dist_right = x_end - 1 - x;
            // t goes 0→1 over the last SMOOTH_WIDTH px
            let t = if dist_right < SMOOTH_WIDTH && i + 1 < palette.len() {
                1.0 - (dist_right as f32) / (SMOOTH_WIDTH as f32)
            } else {
                0.0
            };

            let blended = Rgba([
                lerp(rgb_cur[0], rgb_next[0], t),
                lerp(rgb_cur[1], rgb_next[1], t),
                lerp(rgb_cur[2], rgb_next[2], t),
                255,
            ]);

            for y in 0..h {
                img.put_pixel(x, y, blended);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (w, h) = (3840, 2160);

    let mut img = RgbaImage::new(w, h);
    draw_bear_flag(&mut img, &PALETTE, w / PALETTE.len() as u32, h);

    // Make the bear paw larger and more prominent (1/3 of height instead of 1/4)
    let paw_target_size = h / 3;
    let bear_paw = get_svg_as_rgba(BEAR_PAW_SVG, paw_target_size)?;

    let paw_w = bear_paw.width() as i32;
    let paw_h = bear_paw.height() as i32;

    // Center the paw horizontally, position near bottom with some margin
    let center_x = (w as i32 - paw_w) / 2;
    let bottom_margin = h / 8;
    let center_y = h as i32 - paw_h as i32 - bottom_margin as i32;

    // Draw paw with shadow for better visibility
    draw_paw_with_shadow(&mut img, &bear_paw, center_x, center_y, 8, 12);

    img.save("bear_flag.png")?;
    Ok(())
}
