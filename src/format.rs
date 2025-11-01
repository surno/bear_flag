//! Output format support for flag images

use std::path::Path;

/// Supported output image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PNG format (lossless, supports transparency)
    Png,
    /// JPEG format (lossy, smaller file size)
    Jpeg,
    /// WebP format (modern, excellent compression)
    WebP,
}

impl OutputFormat {
    /// Determines the output format from a file path extension
    ///
    /// # Arguments
    ///
    /// * `path` - File path to extract extension from
    ///
    /// # Errors
    ///
    /// Returns `FlagError::FormatDetection` if the extension cannot be determined
    /// or is not supported
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, crate::error::FlagError> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| crate::error::FlagError::FormatDetection(
                format!("No file extension in path: {}", path.display())
            ))?;

        match ext.to_lowercase().as_str() {
            "png" => Ok(OutputFormat::Png),
            "jpg" | "jpeg" => Ok(OutputFormat::Jpeg),
            "webp" => Ok(OutputFormat::WebP),
            other => Err(crate::error::FlagError::FormatDetection(
                format!("Unsupported file extension: .{}", other)
            )),
        }
    }

    /// Returns the default file extension for this format
    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::WebP => "webp",
        }
    }

    /// Saves an RGBA image in this format
    ///
    /// # Arguments
    ///
    /// * `img` - The image buffer to save
    /// * `path` - Output file path
    /// * `_quality` - Quality setting (0-100, reserved for future lossy format support)
    ///
    /// # Errors
    ///
    /// Returns `FlagError::ImageSave` if the image cannot be saved
    pub fn save_image(
        self,
        img: &image::RgbaImage,
        path: &str,
        _quality: u8,
    ) -> Result<(), crate::error::FlagError> {
        match self {
            OutputFormat::Png => {
                img.save(path).map_err(|e| crate::error::FlagError::ImageSave {
                    path: path.to_string(),
                    source: e,
                })
            }
            OutputFormat::Jpeg => {
                // Convert RGBA to RGB for JPEG (no alpha channel)
                let rgb_img = image::DynamicImage::ImageRgba8(img.clone())
                    .into_rgb8();
                rgb_img
                    .save_with_format(path, image::ImageFormat::Jpeg)
                    .map_err(|e| crate::error::FlagError::ImageSave {
                        path: path.to_string(),
                        source: e,
                    })
            }
            OutputFormat::WebP => {
                // WebP support requires encoding
                // For now, fall back to PNG since image crate WebP support may vary
                // In production, could use webp crate for better support
                img.save(path).map_err(|e| crate::error::FlagError::ImageSave {
                    path: path.to_string(),
                    source: e,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_path_png() {
        let format = OutputFormat::from_path("test.png").unwrap();
        assert_eq!(format, OutputFormat::Png);
    }

    #[test]
    fn test_format_from_path_jpeg() {
        let format = OutputFormat::from_path("test.jpg").unwrap();
        assert_eq!(format, OutputFormat::Jpeg);
        let format = OutputFormat::from_path("test.JPEG").unwrap();
        assert_eq!(format, OutputFormat::Jpeg);
    }

    #[test]
    fn test_format_from_path_webp() {
        let format = OutputFormat::from_path("test.webp").unwrap();
        assert_eq!(format, OutputFormat::WebP);
    }

    #[test]
    fn test_format_from_path_invalid() {
        assert!(OutputFormat::from_path("test.txt").is_err());
        assert!(OutputFormat::from_path("test").is_err());
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::Jpeg.extension(), "jpg");
        assert_eq!(OutputFormat::WebP.extension(), "webp");
    }
}
