//! Gay Bear Flag Web Service
//!
//! Exposes an HTTP API that renders the gay bear pride flag with smooth color
//! transitions and an overlaid bear paw emblem. The service generates the flag
//! on demand and streams the encoded image bytes to the caller.

use std::{env, net::SocketAddr};

use axum::{
    extract::Query,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
    DynamicImage, ExtendedColorType, ImageEncoder, Rgba, RgbaImage,
};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::net::TcpListener;

/// Embedded bear paw icon sourced from `assets/bear_paw.svg`.
const BEAR_PAW_SVG: &[u8] = include_bytes!("assets/bear_paw.svg");

/// Bear pride palette sampled for smooth horizontal gradients.
const BEAR_PALETTE: [u32; 14] = [
    0xC02A01, 0xF1500A, 0xFB7D22, 0xFA9C3C, 0xE6B75D, 0xF0C578, 0xE3C790, 0xBD7B41, 0x89491D,
    0x4D0509, 0x380605, 0x290A06, 0x1C0808, 0x150705,
];

/// Width used for blending adjacent stripes.
const SMOOTH_WIDTH: u32 = 16;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let router = build_router();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service()).await
}

fn build_router() -> Router {
    Router::new()
        .route("/health", get(healthcheck))
        .route("/flag", get(flag_handler))
}

async fn healthcheck() -> &'static str {
    "ok"
}

async fn flag_handler(Query(params): Query<FlagRequest>) -> Result<Response, AppError> {
    let config = params.into_config()?;
    let image_bytes = generate_flag(&config)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(config.format.as_content_type()),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "inline; filename=beardles-flag-{}x{}.{}",
            config.width,
            config.height,
            config.format.file_extension()
        ))
        .map_err(|_| AppError::InvalidRequest("Failed to set response headers".to_string()))?,
    );

    Ok((headers, image_bytes).into_response())
}

/// Supported device presets for common wallpaper dimensions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    #[serde(rename = "iphone-14-pro-max")]
    Iphone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    #[serde(rename = "iphone-14-pro")]
    Iphone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    #[serde(rename = "iphone-14")]
    Iphone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    #[serde(rename = "iphone-se")]
    IphoneSe,
    /// iPad Pro 12.9" - 2732 x 2048 (landscape)
    #[serde(rename = "ipad-pro-12.9")]
    IpadPro129,
    /// iPad Pro 11" - 2388 x 1668 (landscape)
    #[serde(rename = "ipad-pro-11")]
    IpadPro11,
    /// iPad Air 10.9" - 2360 x 1640 (landscape)
    #[serde(rename = "ipad-air")]
    IpadAir,
    /// Android QHD - 2560 x 1440
    #[serde(rename = "android-qhd")]
    AndroidQhd,
    /// Android FHD - 1920 x 1080
    #[serde(rename = "android-fhd")]
    AndroidFhd,
    /// Desktop 4K - 3840 x 2160
    #[serde(rename = "desktop-4k")]
    Desktop4k,
    /// Desktop 1440p - 2560 x 1440
    #[serde(rename = "desktop-1440p")]
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    #[serde(rename = "desktop-1080p")]
    Desktop1080p,
}

impl From<DevicePreset> for (u32, u32) {
    fn from(preset: DevicePreset) -> Self {
        match preset {
            DevicePreset::Iphone14ProMax => (2796, 1290),
            DevicePreset::Iphone14Pro => (2556, 1179),
            DevicePreset::Iphone14 => (2532, 1170),
            DevicePreset::IphoneSe => (1334, 750),
            DevicePreset::IpadPro129 => (2732, 2048),
            DevicePreset::IpadPro11 => (2388, 1668),
            DevicePreset::IpadAir => (2360, 1640),
            DevicePreset::AndroidQhd => (2560, 1440),
            DevicePreset::AndroidFhd => (1920, 1080),
            DevicePreset::Desktop4k => (3840, 2160),
            DevicePreset::Desktop1440p => (2560, 1440),
            DevicePreset::Desktop1080p => (1920, 1080),
        }
    }
}

impl DevicePreset {
    /// Returns a human-readable name for the device preset.
    pub fn display_name(self) -> &'static str {
        match self {
            DevicePreset::Iphone14ProMax => "iPhone 14/13/12 Pro Max",
            DevicePreset::Iphone14Pro => "iPhone 14/13/12 Pro",
            DevicePreset::Iphone14 => "iPhone 14/13/12",
            DevicePreset::IphoneSe => "iPhone SE (3rd gen)",
            DevicePreset::IpadPro129 => "iPad Pro 12.9\"",
            DevicePreset::IpadPro11 => "iPad Pro 11\"",
            DevicePreset::IpadAir => "iPad Air 10.9\"",
            DevicePreset::AndroidQhd => "Android QHD",
            DevicePreset::AndroidFhd => "Android FHD",
            DevicePreset::Desktop4k => "Desktop 4K",
            DevicePreset::Desktop1440p => "Desktop 1440p",
            DevicePreset::Desktop1080p => "Desktop 1080p",
        }
    }
}

/// Output image formats exposed by the HTTP API.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// PNG format (supports transparency).
    Png,
    /// JPEG format (opaque only).
    Jpeg,
    /// WebP format (supports transparency).
    Webp,
}

impl OutputFormat {
    fn as_content_type(self) -> &'static str {
        match self {
            OutputFormat::Png => "image/png",
            OutputFormat::Jpeg => "image/jpeg",
            OutputFormat::Webp => "image/webp",
        }
    }

    fn supports_transparency(self) -> bool {
        matches!(self, OutputFormat::Png | OutputFormat::Webp)
    }

    fn file_extension(self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Webp => "webp",
        }
    }
}

/// Configuration for generating a bear pride flag image.
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Target width in pixels.
    pub width: u32,
    /// Target height in pixels.
    pub height: u32,
    /// Output encoding format.
    pub format: OutputFormat,
    /// Paw size as a fraction of flag height.
    pub paw_size_ratio: f32,
    /// Whether the paw should be centered (`true`) or bottom-left (`false`).
    pub center_paw: bool,
    /// Whether the background should be transparent.
    pub transparent: bool,
}

impl FlagConfig {
    /// Builds a configuration using preset dimensions with sensible defaults.
    pub fn from_preset(preset: DevicePreset) -> Self {
        let (width, height) = preset.into();
        Self {
            width,
            height,
            format: OutputFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        }
    }

    /// Validates that configuration values are within supported bounds.
    ///
    /// # Errors
    ///
    /// Returns [`FlagError::InvalidConfig`] when width or height is zero, when
    /// the paw ratio falls outside `0.01..=1.0`, or when transparency is
    /// requested for a format that does not support alpha channels.
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
        if self.transparent && !self.format.supports_transparency() {
            return Err(FlagError::InvalidConfig(
                "Transparent backgrounds require PNG or WebP output".to_string(),
            ));
        }
        Ok(())
    }
}

/// Structured request payload accepted by `/flag`.
#[derive(Debug, Deserialize)]
struct FlagRequest {
    preset: Option<DevicePreset>,
    width: Option<u32>,
    height: Option<u32>,
    paw_size: Option<f32>,
    paw_position: Option<PawPosition>,
    transparent: Option<bool>,
    format: Option<OutputFormat>,
}

impl FlagRequest {
    fn into_config(self) -> Result<FlagConfig, AppError> {
        let preset = self.preset.unwrap_or(DevicePreset::Desktop4k);
        let mut config = FlagConfig::from_preset(preset);

        match (self.width, self.height) {
            (Some(width), Some(height)) => {
                config.width = width;
                config.height = height;
            }
            (None, None) => {}
            _ => {
                return Err(AppError::InvalidRequest(
                    "Both width and height must be provided when overriding preset dimensions"
                        .to_string(),
                ));
            }
        }

        if let Some(paw_size) = self.paw_size {
            config.paw_size_ratio = paw_size;
        }
        if let Some(position) = self.paw_position {
            config.center_paw = matches!(position, PawPosition::Center);
        }
        if let Some(transparent) = self.transparent {
            config.transparent = transparent;
        }
        if let Some(format) = self.format {
            config.format = format;
        }

        match config.validate() {
            Ok(()) => Ok(config),
            Err(error) => Err(AppError::from(error)),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PawPosition {
    Center,
    BottomLeft,
}

/// Flag generation failures surfaced to callers.
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Image encoding failed: {source}")]
    ImageEncoding {
        #[from]
        source: image::ImageError,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Application-level errors mapped to HTTP responses.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Flag generation failed: {source}")]
    FlagInternal {
        #[source]
        source: FlagError,
    },
}

impl From<FlagError> for AppError {
    fn from(error: FlagError) -> Self {
        match error {
            FlagError::InvalidConfig(message) => AppError::InvalidRequest(message),
            other => AppError::FlagInternal { source: other },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::FlagInternal { source } => {
                (StatusCode::INTERNAL_SERVER_ERROR, source.to_string())
            }
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}

/// Renders an SVG to an RGBA image buffer scaled to `target_size`.
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

    image::ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| FlagError::BufferCreation {
            width: pixmap.width(),
            height: pixmap.height(),
        })
}

/// Linear interpolation between two 8-bit channel values.
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t).mul_add(a as f32, t * b as f32)).round() as u8
}

/// Paints horizontal bear pride stripes with smooth blending transitions.
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

/// Alpha-composites the paw icon over the flag stripes.
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

fn encode_image(image: RgbaImage, format: OutputFormat) -> Result<Vec<u8>, FlagError> {
    match format {
        OutputFormat::Png => encode_png(image),
        OutputFormat::Jpeg => encode_jpeg(image),
        OutputFormat::Webp => encode_webp(image),
    }
}

fn encode_png(image: RgbaImage) -> Result<Vec<u8>, FlagError> {
    let width = image.width();
    let height = image.height();
    let raw = image.into_raw();

    let mut buffer = Vec::new();
    PngEncoder::new(&mut buffer)
        .write_image(&raw, width, height, ExtendedColorType::Rgba8)
        .map_err(|source| FlagError::ImageEncoding { source })?;
    Ok(buffer)
}

fn encode_jpeg(image: RgbaImage) -> Result<Vec<u8>, FlagError> {
    let dynamic = DynamicImage::ImageRgba8(image);
    let rgb_image = dynamic.into_rgb8();

    let mut buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 90);
    encoder
        .encode_image(&DynamicImage::ImageRgb8(rgb_image))
        .map_err(|source| FlagError::ImageEncoding { source })?;
    Ok(buffer)
}

fn encode_webp(image: RgbaImage) -> Result<Vec<u8>, FlagError> {
    let width = image.width();
    let height = image.height();
    let raw = image.into_raw();

    let mut buffer = Vec::new();
    WebPEncoder::new_lossless(&mut buffer)
        .encode(&raw, width, height, ExtendedColorType::Rgba8)
        .map_err(|source| FlagError::ImageEncoding { source })?;
    Ok(buffer)
}

/// Generates a bear pride flag and returns the encoded image bytes.
///
/// # Errors
///
/// Returns [`FlagError`] variants when configuration validation fails, when
/// the paw SVG cannot be rendered, or when encoding the output image fails.
pub fn generate_flag(config: &FlagConfig) -> Result<Vec<u8>, FlagError> {
    config.validate()?;

    let mut img = if config.transparent {
        RgbaImage::from_pixel(config.width, config.height, Rgba([0, 0, 0, 0]))
    } else {
        RgbaImage::new(config.width, config.height)
    };

    let stripe_width =
        ((config.width + BEAR_PALETTE.len() as u32 - 1) / BEAR_PALETTE.len() as u32).max(1);
    draw_bear_stripes(&mut img, &BEAR_PALETTE, stripe_width, config.height);

    let paw_size = ((config.height as f32 * config.paw_size_ratio).round() as u32).max(1);
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

    encode_image(img, config.format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tower::ServiceExt;

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
            format: OutputFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_paw_ratio() {
        let config = FlagConfig {
            width: 100,
            height: 100,
            format: OutputFormat::Png,
            paw_size_ratio: 1.5,
            center_paw: true,
            transparent: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_transparent_jpeg_rejected() {
        let config = FlagConfig {
            width: 100,
            height: 100,
            format: OutputFormat::Jpeg,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = FlagConfig::from_preset(DevicePreset::Desktop1080p);
        assert!(config.validate().is_ok());
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
        assert!(blended_pixel[0] > 0);
        assert!(blended_pixel[2] > 0);
    }

    #[test]
    fn test_generate_flag_returns_bytes() {
        let config = FlagConfig {
            width: 140,
            height: 80,
            format: OutputFormat::Png,
            paw_size_ratio: 0.3,
            center_paw: true,
            transparent: false,
        };

        let bytes = generate_flag(&config).expect("Flag generation failed");
        assert!(!bytes.is_empty());

        let reader = image::ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .expect("Failed to guess format");
        let decoded = reader.decode().expect("Failed to decode image");
        assert_eq!(decoded.width(), 140);
        assert_eq!(decoded.height(), 80);
    }

    #[tokio::test]
    async fn test_flag_endpoint_returns_png() {
        let app = super::build_router();

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/flag?preset=desktop-1080p&format=png")
                    .body(axum::body::Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content-type header");
        assert_eq!(content_type, "image/png");
    }

    #[test]
    fn test_device_preset_dimensions() {
        let (width, height) = DevicePreset::Iphone14ProMax.into();
        assert_eq!((width, height), (2796, 1290));
    }

    #[test]
    fn test_device_preset_display_names() {
        assert_eq!(DevicePreset::Desktop4k.display_name(), "Desktop 4K");
        assert_eq!(DevicePreset::IpadAir.display_name(), "iPad Air 10.9\"");
    }
}
