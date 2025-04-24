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

fn main() {
    let (w, h) = (3840, 2160);

    let mut img = RgbaImage::new(w, h);
    draw_bear_flag(&mut img, &PALETTE, w / PALETTE.len() as u32, h);

    let bear_paw = get_svg_as_rgba(BEAR_PAW_SVG, h / 4).unwrap();

    let bear_paw_height = bear_paw.height();

    // Draw the bear paw onto the flag (bottom‑left) while honoring transparency
    for (x, y, pixel) in bear_paw.enumerate_pixels() {
        if pixel[3] != 0 {
            img.put_pixel(x, y + h - bear_paw_height, *pixel);
        }
    }

    img.save("bear_flag.png").unwrap();
}
