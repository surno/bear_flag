//! Gay Bear Flag Generator Web Service
//!
//! Serves an Axum-based web API that generates a high-quality gay bear pride
//! flag with smooth gradients and an SVG paw overlay.

use std::io::Cursor;
use std::net::SocketAddr;

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use image::{DynamicImage, ImageBuffer, ImageFormat, RgbImage, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Embeds assets/bear_paw.svg directly into the binary.
const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

/// Traditional bear pride palette representing the community's diversity.
const BEAR_PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

/// Number of pixels across which adjacent stripes blend for smoother gradients.
const SMOOTH_WIDTH: u32 = 16;

/// Errors that can occur during flag generation or request handling.
#[derive(Error, Debug)]
pub enum FlagError {
    /// The embedded SVG could not be parsed or rendered.
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    /// Creating an image buffer with the requested dimensions failed.
    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    /// Encoding the generated image into the requested format failed.
    #[error("Failed to encode image: {source}")]
    ImageEncode { source: image::ImageError },

    /// The caller supplied an invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Supported device presets for convenience sizing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
    /// Converts a device preset to its `(width, height)` dimensions.
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
    /// Returns a human-readable name for the device preset.
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

/// Output image formats supported by the API.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// PNG format (supports transparency).
    Png,
    /// JPEG format (opaque only).
    #[serde(alias = "jpg")]
    #[serde(alias = "jpeg")]
    Jpeg,
    /// WebP format (supports transparency).
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

/// Configuration for flag generation.
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Output image width in pixels.
    pub width: u32,
    /// Output image height in pixels.
    pub height: u32,
    /// Output image format.
    pub output_format: ImageFormat,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0).
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw (otherwise bottom-left).
    pub center_paw: bool,
    /// Whether to start from a transparent canvas.
    pub transparent: bool,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_format: ImageFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        }
    }
}

impl FlagConfig {
    /// Creates a configuration from a device preset.
    pub fn from_preset(preset: DevicePreset) -> Self {
        let (width, height) = preset.into();
        Self {
            width,
            height,
            output_format: ImageFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        }
    }

    /// Validates the configuration parameters.
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

        if self.transparent && self.output_format == ImageFormat::Jpeg {
            return Err(FlagError::InvalidConfig(
                "JPEG format does not support transparency".to_string(),
            ));
        }

        Ok(())
    }
}

/// Holds the encoded image bytes and metadata for HTTP responses.
pub struct FlagImage {
    bytes: Vec<u8>,
    format: ImageFormat,
}

impl FlagImage {
    /// Returns the encoded bytes for testing or further handling.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the image format associated with the encoded bytes.
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    fn mime_type(&self) -> &'static str {
        match self.format {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
            _ => "application/octet-stream",
        }
    }
}

impl IntoResponse for FlagImage {
    fn into_response(self) -> Response {
        let mime = self.mime_type();
        let bytes = self.bytes;
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "no-store"),
                (header::PRAGMA, "no-cache"),
            ],
            bytes,
        )
            .into_response()
    }
}

impl IntoResponse for FlagError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            error: String,
        }

        let status = match self {
            FlagError::InvalidConfig(_) => StatusCode::BAD_REQUEST,
            FlagError::SvgParse(_) => StatusCode::UNPROCESSABLE_ENTITY,
            FlagError::BufferCreation { .. } | FlagError::ImageEncode { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = Json(ErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

/// Describes the incoming JSON payload for the flag generation endpoint.
#[derive(Debug, Deserialize)]
struct FlagRequest {
    /// Optional device preset to base dimensions on.
    #[serde(default)]
    preset: Option<DevicePreset>,
    /// Explicit width override in pixels.
    #[serde(default)]
    width: Option<u32>,
    /// Explicit height override in pixels.
    #[serde(default)]
    height: Option<u32>,
    /// Desired output format.
    #[serde(default)]
    format: Option<OutputFormat>,
    /// Paw size ratio override (defaults to 0.35).
    #[serde(default = "default_paw_size_ratio")]
    paw_size_ratio: f32,
    /// Center paw flag (defaults to true).
    #[serde(default = "default_center_paw")]
    center_paw: bool,
    /// Transparent background toggle.
    #[serde(default)]
    transparent: bool,
}

impl Default for FlagRequest {
    fn default() -> Self {
        Self {
            preset: None,
            width: None,
            height: None,
            format: None,
            paw_size_ratio: default_paw_size_ratio(),
            center_paw: default_center_paw(),
            transparent: false,
        }
    }
}

fn default_paw_size_ratio() -> f32 {
    0.35
}

fn default_center_paw() -> bool {
    true
}

impl FlagRequest {
    fn into_config(self) -> Result<FlagConfig, FlagError> {
        let mut config = match self.preset {
            Some(preset) => FlagConfig::from_preset(preset),
            None => FlagConfig::default(),
        };

        if let Some(width) = self.width {
            config.width = width;
        }

        if let Some(height) = self.height {
            config.height = height;
        }

        if let Some(format) = self.format {
            config.output_format = format.into();
        }

        config.paw_size_ratio = self.paw_size_ratio;
        config.center_paw = self.center_paw;
        config.transparent = self.transparent;

        config.validate()?;
        Ok(config)
    }
}

/// Generates the complete gay bear pride flag and returns the encoded image bytes.
///
/// # Errors
///
/// Returns `FlagError` variants when rendering or encoding fails.
pub fn generate_flag(config: &FlagConfig) -> Result<FlagImage, FlagError> {
    config.validate()?;

    let mut img = if config.transparent {
        RgbaImage::from_pixel(config.width, config.height, Rgba([0, 0, 0, 0]))
    } else {
        RgbaImage::new(config.width, config.height)
    };

    let stripe_width = (config.width / BEAR_PALETTE.len() as u32).max(1);
    draw_bear_stripes(&mut img, &BEAR_PALETTE, stripe_width, config.height);

    let paw_size = (config.height as f32 * config.paw_size_ratio)
        .round()
        .max(1.0) as u32;
    let bear_paw = render_svg_to_rgba(BEAR_PAW_SVG, paw_size)?;

    let (paw_x, paw_y) = if config.center_paw {
        (
            config.width.saturating_sub(bear_paw.width()) / 2,
            config.height.saturating_sub(bear_paw.height()) / 2,
        )
    } else {
        (0, config.height.saturating_sub(bear_paw.height()))
    };

    composite_with_alpha(&mut img, &bear_paw, paw_x, paw_y);

    let image = if config.output_format == ImageFormat::Jpeg {
        let rgb = RgbImage::from_fn(img.width(), img.height(), |x, y| {
            let pixel = img.get_pixel(x, y);
            image::Rgb([pixel[0], pixel[1], pixel[2]])
        });
        DynamicImage::ImageRgb8(rgb)
    } else {
        DynamicImage::ImageRgba8(img)
    };
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, config.output_format)
        .map_err(|source| FlagError::ImageEncode { source })?;

    Ok(FlagImage {
        bytes: buffer.into_inner(),
        format: config.output_format,
    })
}

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

fn render_svg_to_rgba(svg_data: &[u8], target_size: u32) -> Result<RgbaImage, FlagError> {
    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
        .map_err(|e| FlagError::SvgParse(e.to_string()))?;

    let svg_size = tree.size();
    let max_dim = svg_size.width().max(svg_size.height()).max(1.0);
    let scale = target_size as f32 / max_dim;

    let width_px = (svg_size.width() * scale).ceil().max(1.0) as u32;
    let height_px = (svg_size.height() * scale).ceil().max(1.0) as u32;

    let mut pixmap = Pixmap::new(width_px, height_px).ok_or(FlagError::BufferCreation {
        width: width_px,
        height: height_px,
    })?;

    let transform = Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec()).ok_or(
        FlagError::BufferCreation {
            width: pixmap.width(),
            height: pixmap.height(),
        },
    )
}

fn composite_with_alpha(dst: &mut RgbaImage, src: &RgbaImage, offset_x: u32, offset_y: u32) {
    for (src_x, src_y, src_pixel) in src.enumerate_pixels() {
        let dst_x = offset_x + src_x;
        let dst_y = offset_y + src_y;

        if dst_x >= dst.width() || dst_y >= dst.height() {
            continue;
        }

        let src_alpha = src_pixel[3] as f32 / 255.0;

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

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t).mul_add(a as f32, t * b as f32)).round() as u8
}

async fn create_flag(Json(request): Json<FlagRequest>) -> Result<FlagImage, FlagError> {
    let config = request.into_config()?;
    generate_flag(&config)
}

fn app() -> Router {
    Router::new().route("/flags", post(create_flag))
}

#[derive(Error, Debug)]
enum ServerError {
    #[error(transparent)]
    Flag(#[from] FlagError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP server error: {0}")]
    Http(#[from] axum::Error),
    #[error("Invalid socket address: {0}")]
    Address(std::net::AddrParseError),
}

async fn run() -> Result<(), ServerError> {
    let addr = server_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;
    println!("Listening on http://{}", actual_addr);

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn server_addr() -> Result<SocketAddr, ServerError> {
    let raw = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    raw.parse().map_err(ServerError::Address)
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for shutdown signal: {err}");
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Server failed: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use image::ImageFormat;

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
            ..FlagConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_paw_ratio() {
        let config = FlagConfig {
            paw_size_ratio: 1.5,
            ..FlagConfig::default()
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
            output_format: ImageFormat::Png,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: false,
        };

        let result = generate_flag(&config);
        assert!(result.is_ok(), "Flag generation failed: {:?}", result.err());
        let image = result.unwrap();
        assert!(!image.bytes().is_empty());
        assert_eq!(image.format(), ImageFormat::Png);
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
    fn test_device_preset_dimensions() {
        let iphone: (u32, u32) = DevicePreset::IPhone14ProMax.into();
        let desktop: (u32, u32) = DevicePreset::Desktop4K.into();
        assert_eq!(iphone, (2796, 1290));
        assert_eq!(desktop, (3840, 2160));
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
    fn test_device_preset_display_name() {
        assert_eq!(DevicePreset::Desktop4K.display_name(), "Desktop 4K");
    }

    #[test]
    fn test_flag_request_into_config_overrides() {
        let request = FlagRequest {
            preset: Some(DevicePreset::Desktop1080p),
            width: Some(1024),
            height: Some(512),
            format: Some(OutputFormat::WebP),
            paw_size_ratio: 0.4,
            center_paw: false,
            transparent: true,
        };

        let config = request.into_config().expect("config should be valid");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 512);
        assert_eq!(config.output_format, ImageFormat::WebP);
        assert!(!config.center_paw);
        assert!(config.transparent);
    }

    #[tokio::test]
    async fn test_create_flag_endpoint() {
        let response = create_flag(Json(FlagRequest {
            width: Some(320),
            height: Some(180),
            format: Some(OutputFormat::Jpeg),
            ..FlagRequest::default()
        }))
        .await
        .expect("request should succeed");

        assert!(!response.bytes().is_empty());
        assert_eq!(response.format(), ImageFormat::Jpeg);
    }
}
