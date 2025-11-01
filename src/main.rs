//! Gay Bear Flag Generator - Axum Web Service
//!
//! Generates high-quality gay bear pride flags with smooth color gradients
//! and a centered bear paw overlay. Exposed as a REST API for programmatic access.

use axum::{
    extract::Json,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::usvg::Transform;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use validator::Validate;

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

/// Errors that can occur during flag generation
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Failed to encode image: {0}")]
    ImageEncode(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// API error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

impl IntoResponse for FlagError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            FlagError::SvgParse(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FlagError::BufferCreation { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            FlagError::ImageEncode(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            FlagError::InvalidConfig(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
            details: None,
        });

        (status, body).into_response()
    }
}

/// Device preset configurations with appropriate dimensions for wallpapers
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    Iphone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    Iphone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    Iphone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    IphoneSe,
    /// iPad Pro 12.9" - 2732 x 2048 (landscape)
    IpadPro129,
    /// iPad Pro 11" - 2388 x 1668 (landscape)
    IpadPro11,
    /// iPad Air 10.9" - 2360 x 1640 (landscape)
    IpadAir,
    /// Android QHD - 2560 x 1440
    AndroidQhd,
    /// Android FHD - 1920 x 1080
    AndroidFhd,
    /// Desktop 4K - 3840 x 2160
    Desktop4k,
    /// Desktop 1440p - 2560 x 1440
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    Desktop1080p,
}

impl From<DevicePreset> for (u32, u32) {
    /// Converts a device preset to (width, height) dimensions
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

/// Output image format
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// PNG format (supports transparency)
    Png,
    /// JPEG format (no transparency support)
    Jpeg,
    /// WebP format (supports transparency)
    Webp,
}

impl From<OutputFormat> for ImageFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::Webp => ImageFormat::WebP,
        }
    }
}

impl OutputFormat {
    /// Returns the MIME type for this format
    fn mime_type(self) -> &'static str {
        match self {
            OutputFormat::Png => "image/png",
            OutputFormat::Jpeg => "image/jpeg",
            OutputFormat::Webp => "image/webp",
        }
    }
}

/// API request for generating a flag
#[derive(Debug, Deserialize, Validate)]
pub struct GenerateFlagRequest {
    /// Device preset to use for dimensions (optional if width/height provided)
    #[serde(default)]
    pub preset: Option<DevicePreset>,

    /// Custom width in pixels (overrides preset)
    #[validate(range(min = 100, max = 10000))]
    pub width: Option<u32>,

    /// Custom height in pixels (overrides preset)
    #[validate(range(min = 100, max = 10000))]
    pub height: Option<u32>,

    /// Output image format (defaults to PNG)
    #[serde(default = "default_format")]
    pub format: OutputFormat,

    /// Size of the bear paw as a fraction of flag height (0.01-1.0)
    #[validate(range(min = 0.01, max = 1.0))]
    #[serde(default = "default_paw_size")]
    pub paw_size_ratio: f32,

    /// Whether to center the bear paw (true) or place in bottom-left (false)
    #[serde(default = "default_center_paw")]
    pub center_paw: bool,

    /// Whether to use transparent background (only for PNG/WebP)
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

/// Configuration for flag generation (internal)
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Output image width in pixels
    pub width: u32,
    /// Output image height in pixels
    pub height: u32,
    /// Image format for output
    pub output_format: ImageFormat,
    /// Size of the bear paw as a fraction of flag height (0.0-1.0)
    pub paw_size_ratio: f32,
    /// Whether to center the bear paw vertically and horizontally
    pub center_paw: bool,
    /// Whether to use transparent background (only for formats that support it)
    pub transparent: bool,
}

impl FlagConfig {
    /// Creates a configuration from an API request
    ///
    /// # Errors
    ///
    /// Returns `FlagError::InvalidConfig` if dimensions cannot be determined
    pub fn from_request(req: &GenerateFlagRequest) -> Result<Self, FlagError> {
        let (width, height) = match (req.width, req.height, req.preset) {
            (Some(w), Some(h), _) => (w, h),
            (None, None, Some(preset)) => preset.into(),
            (None, None, None) => {
                return Err(FlagError::InvalidConfig(
                    "Must provide either preset or both width and height".to_string(),
                ))
            }
            _ => {
                return Err(FlagError::InvalidConfig(
                    "Must provide both width and height, or use a preset".to_string(),
                ))
            }
        };

        let format = req.format.into();

        // Disable transparency for JPEG
        let transparent = req.transparent && format != ImageFormat::Jpeg;

        Ok(Self {
            width,
            height,
            output_format: format,
            paw_size_ratio: req.paw_size_ratio,
            center_paw: req.center_paw,
            transparent,
        })
    }

    /// Validates the configuration
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

    let mut pixmap = Pixmap::new(width_px, height_px).ok_or(FlagError::BufferCreation {
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

/// Generates the complete gay bear pride flag and returns the image bytes
///
/// Creates a flag with smooth color transitions and a bear paw overlay.
///
/// # Arguments
///
/// * `config` - Configuration specifying dimensions and styling
///
/// # Errors
///
/// Returns errors if SVG rendering fails, image buffer creation fails,
/// or the image cannot be encoded.
pub fn generate_flag(config: &FlagConfig) -> Result<Vec<u8>, FlagError> {
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
        // Bottom-left positioning
        let x = 0;
        let y = config.height.saturating_sub(bear_paw.height());
        (x, y)
    };

    composite_with_alpha(&mut img, &bear_paw, paw_x, paw_y);

    // Encode image to bytes
    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, config.output_format)
        .map_err(|e| FlagError::ImageEncode(e.to_string()))?;

    Ok(buffer.into_inner())
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "bear-flag-generator",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Generate flag endpoint
async fn generate_flag_handler(
    Json(req): Json<GenerateFlagRequest>,
) -> Result<impl IntoResponse, FlagError> {
    // Validate request
    req.validate()
        .map_err(|e| FlagError::InvalidConfig(e.to_string()))?;

    info!(
        "Generating flag with preset={:?}, width={:?}, height={:?}, format={:?}",
        req.preset, req.width, req.height, req.format
    );

    // Create config from request
    let config = FlagConfig::from_request(&req)?;

    // Generate flag
    let image_bytes = generate_flag(&config)?;

    // Return image with appropriate content type
    let content_type = req.format.mime_type();

    Ok(([(header::CONTENT_TYPE, content_type)], image_bytes))
}

/// Creates the Axum application router
fn app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/generate", post(generate_flag_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    info!("Starting Bear Flag Generator API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}", addr);
    info!("Health check: http://{}/health", addr);
    info!("Generate endpoint: POST http://{}/generate", addr);

    axum::serve(listener, app())
        .await
        .expect("Server failed to start");
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
            output_format: ImageFormat::Png,
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
            output_format: ImageFormat::Png,
            paw_size_ratio: 1.5,
            center_paw: true,
            transparent: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = FlagConfig {
            width: 100,
            height: 100,
            output_format: ImageFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        };
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

        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "Generated image has no data");
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
    fn test_device_preset_conversions() {
        let (w, h) = DevicePreset::Iphone14ProMax.into();
        assert_eq!(w, 2796);
        assert_eq!(h, 1290);

        let (w, h) = DevicePreset::Desktop4k.into();
        assert_eq!(w, 3840);
        assert_eq!(h, 2160);
    }

    #[test]
    fn test_config_from_request_preset() {
        let req = GenerateFlagRequest {
            preset: Some(DevicePreset::Desktop1080p),
            width: None,
            height: None,
            format: OutputFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        };

        let config = FlagConfig::from_request(&req).unwrap();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_config_from_request_custom_dimensions() {
        let req = GenerateFlagRequest {
            preset: None,
            width: Some(800),
            height: Some(600),
            format: OutputFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        };

        let config = FlagConfig::from_request(&req).unwrap();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn test_config_from_request_no_dimensions() {
        let req = GenerateFlagRequest {
            preset: None,
            width: None,
            height: None,
            format: OutputFormat::Png,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: false,
        };

        assert!(FlagConfig::from_request(&req).is_err());
    }

    #[test]
    fn test_jpeg_disables_transparency() {
        let req = GenerateFlagRequest {
            preset: Some(DevicePreset::Desktop1080p),
            width: None,
            height: None,
            format: OutputFormat::Jpeg,
            paw_size_ratio: 0.35,
            center_paw: true,
            transparent: true,
        };

        let config = FlagConfig::from_request(&req).unwrap();
        assert!(!config.transparent, "JPEG should disable transparency");
    }

    #[test]
    fn test_output_format_mime_types() {
        assert_eq!(OutputFormat::Png.mime_type(), "image/png");
        assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(OutputFormat::Webp.mime_type(), "image/webp");
    }
}
