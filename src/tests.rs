//! Test suite for bear flag generation

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use image::Rgba;
use image::RgbaImage;
use tower::ServiceExt;

use crate::config::{DevicePreset, FlagConfig, OutputFormat};
use crate::constants::BEAR_PAW_SVG;
use crate::image::flag::generate_flag_bytes;
use crate::image::rendering::{composite_with_alpha, lerp_u8, render_svg_to_rgba};
use crate::router::create_router;

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
fn test_config_validation_too_large() {
    let config = FlagConfig {
        width: 20000,
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
fn test_device_preset_desktop_4k() {
    let (width, height) = DevicePreset::Desktop4K.into();
    assert_eq!(width, 3840);
    assert_eq!(height, 2160);
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
fn test_output_format_mime_types() {
    assert_eq!(OutputFormat::Png.mime_type(), "image/png");
    assert_eq!(OutputFormat::Jpeg.mime_type(), "image/jpeg");
    assert_eq!(OutputFormat::WebP.mime_type(), "image/webp");
}

#[tokio::test]
async fn test_health_endpoint() {
    let response = create_router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_flag_endpoint_default() {
    let response = create_router()
        .oneshot(Request::builder().uri("/flag").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/png"
    );
}

#[tokio::test]
async fn test_flag_endpoint_with_preset() {
    let response = create_router()
        .oneshot(
            Request::builder()
                .uri("/flag?preset=iphone-14-pro-max")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_flag_endpoint_with_custom_dimensions() {
    let response = create_router()
        .oneshot(
            Request::builder()
                .uri("/flag?width=640&height=480")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
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

#[tokio::test]
async fn test_flag_endpoint_jpeg_format() {
    use axum::body::to_bytes;

    let response = create_router()
        .oneshot(
            Request::builder()
                .uri("/flag?format=jpeg&width=320&height=240")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let headers = response.headers().clone();

    if status != StatusCode::OK {
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        panic!("Expected 200 OK, got {}, body: {}", status, body_str);
    }

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/jpeg");
}

#[tokio::test]
async fn test_flag_endpoint_invalid_dimensions() {
    let response = create_router()
        .oneshot(
            Request::builder()
                .uri("/flag?width=0&height=100")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
