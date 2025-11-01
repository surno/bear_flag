//! Beardles Flag Generator Web Service
//!
//! Axum web service that generates high-quality beardles (gay bear pride) flags
//! with smooth color gradients and a centered bear paw overlay via HTTP API.

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::usvg::Transform;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::CorsLayer;

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

/// Application state
#[derive(Clone)]
pub struct AppState {
    /// Maximum allowed image dimensions to prevent excessive memory usage
    pub max_dimensions: (u32, u32),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            max_dimensions: (7680, 4320), // 8K maximum
        }
    }
}

/// Errors that can occur during flag generation or API requests
#[derive(Error, Debug)]
pub enum FlagError {
    #[error("Failed to parse SVG data: {0}")]
    SvgParse(String),

    #[error("Failed to create image buffer with dimensions {width}x{height}")]
    BufferCreation { width: u32, height: u32 },

    #[error("Failed to encode image: {0}")]
    ImageEncode(#[from] image::ImageError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Image dimensions exceed maximum allowed size")]
    DimensionsTooLarge,
}

impl IntoResponse for FlagError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            FlagError::InvalidConfig(msg) | FlagError::SvgParse(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            FlagError::BufferCreation { width, height } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Failed to create image buffer with dimensions {}x{}",
                    width, height
                ),
            ),
            FlagError::ImageEncode(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            FlagError::DimensionsTooLarge => (
                StatusCode::BAD_REQUEST,
                "Image dimensions exceed maximum allowed size".to_string(),
            ),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

/// API request parameters for flag generation
#[derive(Debug, Clone, Deserialize)]
pub struct FlagRequest {
    /// Output image width in pixels (default: 1920)
    #[serde(default = "default_width")]
    pub width: u32,
    /// Output image height in pixels (default: 1080)
    #[serde(default = "default_height")]
    pub height: u32,
    /// Image format for output: "png", "jpg", "jpeg", or "webp" (default: "png")
    #[serde(default = "default_format")]
    pub format: String,
    /// Size of the bear paw as a fraction of flag height (0.01-1.0, default: 0.35)
    #[serde(default = "default_paw_size")]
    pub paw_size: f32,
    /// Whether to center the bear paw (default: true)
    #[serde(default = "default_center_paw")]
    pub center_paw: bool,
    /// Whether to use transparent background (only for PNG/WebP formats, default: false)
    #[serde(default)]
    pub transparent: bool,
}

impl Default for FlagRequest {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            format: default_format(),
            paw_size: default_paw_size(),
            center_paw: default_center_paw(),
            transparent: false,
        }
    }
}

fn default_width() -> u32 {
    1920
}

fn default_height() -> u32 {
    1080
}

fn default_format() -> String {
    "png".to_string()
}

fn default_paw_size() -> f32 {
    0.35
}

fn default_center_paw() -> bool {
    true
}

/// API response containing flag metadata
#[derive(Debug, Serialize)]
pub struct FlagResponse {
    /// Message indicating success
    pub message: String,
    /// Image dimensions
    pub dimensions: (u32, u32),
    /// Image format
    pub format: String,
}

/// Internal configuration for flag generation
#[derive(Debug, Clone)]
struct FlagConfig {
    width: u32,
    height: u32,
    output_format: ImageFormat,
    paw_size_ratio: f32,
    center_paw: bool,
    transparent: bool,
}

impl FlagConfig {
    fn from_request(req: FlagRequest, max_dims: (u32, u32)) -> Result<Self, FlagError> {
        if req.width == 0 || req.height == 0 {
            return Err(FlagError::InvalidConfig(
                "Width and height must be non-zero".to_string(),
            ));
        }

        if req.width > max_dims.0 || req.height > max_dims.1 {
            return Err(FlagError::DimensionsTooLarge);
        }

        if !(0.01..=1.0).contains(&req.paw_size) {
            return Err(FlagError::InvalidConfig(
                "Paw size ratio must be between 0.01 and 1.0".to_string(),
            ));
        }

        let output_format = match req.format.to_lowercase().as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => {
                if req.transparent {
                    return Err(FlagError::InvalidConfig(
                        "JPEG format does not support transparency".to_string(),
                    ));
                }
                ImageFormat::Jpeg
            }
            "webp" => ImageFormat::WebP,
            _ => {
                return Err(FlagError::InvalidConfig(format!(
                    "Unsupported format: {}. Supported formats: png, jpg, jpeg, webp",
                    req.format
                )));
            }
        };

        Ok(Self {
            width: req.width,
            height: req.height,
            output_format,
            paw_size_ratio: req.paw_size,
            center_paw: req.center_paw,
            transparent: req.transparent,
        })
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

/// Generates the complete beardles flag and returns it as image bytes
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
fn generate_flag_bytes(config: &FlagConfig) -> Result<Vec<u8>, FlagError> {
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

    // Encode image to bytes
    let mut bytes = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut bytes);
        img.write_to(&mut cursor, config.output_format)?;
    }

    Ok(bytes)
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "beardles-flag-generator"
    }))
}

/// Generate flag endpoint
///
/// GET /flag - Generates a beardles flag image based on query parameters
/// Returns the image as PNG/JPEG/WebP bytes with appropriate Content-Type header
async fn generate_flag_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlagRequest>,
) -> Result<Response, FlagError> {
    let config = FlagConfig::from_request(params.clone(), state.max_dimensions)?;
    let image_bytes = generate_flag_bytes(&config)?;

    let content_type = match config.output_format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::WebP => "image/webp",
        _ => "image/png",
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        image_bytes,
    )
        .into_response())
}

/// Generate flag metadata endpoint
///
/// GET /flag/info - Returns metadata about the flag generation without the image
async fn flag_info_handler(
    Query(params): Query<FlagRequest>,
) -> Result<Json<FlagResponse>, FlagError> {
    let config = FlagConfig::from_request(params.clone(), (7680, 4320))?;

    Ok(Json(FlagResponse {
        message: "Flag generated successfully".to_string(),
        dimensions: (config.width, config.height),
        format: match config.output_format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::WebP => "webp",
            _ => "png",
        }
        .to_string(),
    }))
}

/// Builds the Axum router with all routes
fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/flag", get(generate_flag_handler))
        .route("/flag/info", get(flag_info_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = Arc::new(AppState::default());
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    tracing::info!("Beardles flag generator service listening on http://0.0.0.0:3000");
    tracing::info!("Endpoints:");
    tracing::info!("  GET /health - Health check");
    tracing::info!("  GET /flag?width=1920&height=1080&format=png - Generate flag image");
    tracing::info!("  GET /flag/info?width=1920&height=1080 - Get flag metadata");

    axum::serve(listener, app)
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
    fn test_flag_config_validation_zero_dimensions() {
        let req = FlagRequest {
            width: 0,
            height: 100,
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_err());
    }

    #[test]
    fn test_flag_config_validation_invalid_paw_ratio() {
        let req = FlagRequest {
            paw_size: 1.5,
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_err());
    }

    #[test]
    fn test_flag_config_validation_dimensions_too_large() {
        let req = FlagRequest {
            width: 10000,
            height: 10000,
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_err());
    }

    #[test]
    fn test_flag_config_validation_valid() {
        let req = FlagRequest {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_ok());
    }

    #[test]
    fn test_generate_flag_bytes_small() {
        let config = FlagConfig {
            width: 140,
            height: 80,
            output_format: ImageFormat::Png,
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
    fn test_flag_config_from_request_jpeg_transparent_error() {
        let req = FlagRequest {
            format: "jpeg".to_string(),
            transparent: true,
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_err());
    }

    #[test]
    fn test_flag_config_from_request_invalid_format() {
        let req = FlagRequest {
            format: "gif".to_string(),
            ..Default::default()
        };
        assert!(FlagConfig::from_request(req, (7680, 4320)).is_err());
    }
}
