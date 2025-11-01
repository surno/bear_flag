//! Bear Flag Generator CLI
//!
//! Command-line interface for generating gay bear pride flags
//! with smooth color gradients and bear paw overlays.

use bear_flag::{generate_flag, DevicePreset, FlagConfig, FlagError, OutputFormat};
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;

/// Output format for generated flags
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFormat {
    /// PNG format (lossless, supports transparency)
    Png,
    /// JPEG format (lossy, smaller file size)
    Jpeg,
    /// WebP format (modern, excellent compression)
    WebP,
}

impl From<CliFormat> for OutputFormat {
    fn from(f: CliFormat) -> Self {
        match f {
            CliFormat::Png => OutputFormat::Png,
            CliFormat::Jpeg => OutputFormat::Jpeg,
            CliFormat::WebP => OutputFormat::WebP,
        }
    }
}

/// CLI-friendly device preset enum
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliDevicePreset {
    /// iPhone 14/13/12 Pro Max - 2796 x 1290 (landscape)
    #[value(name = "iphone-14-pro-max")]
    IPhone14ProMax,
    /// iPhone 14/13/12 Pro - 2556 x 1179 (landscape)
    #[value(name = "iphone-14-pro")]
    IPhone14Pro,
    /// iPhone 14/13/12 - 2532 x 1170 (landscape)
    #[value(name = "iphone-14")]
    IPhone14,
    /// iPhone SE (3rd gen) - 1334 x 750 (landscape)
    #[value(name = "iphone-se")]
    IPhoneSE,
    /// Desktop 4K - 3840 x 2160
    #[value(name = "desktop-4k")]
    Desktop4K,
    /// Desktop 1440p - 2560 x 1440
    #[value(name = "desktop-1440p")]
    Desktop1440p,
    /// Desktop 1080p - 1920 x 1080
    #[value(name = "desktop-1080p")]
    Desktop1080p,
}

impl From<CliDevicePreset> for DevicePreset {
    fn from(p: CliDevicePreset) -> Self {
        match p {
            CliDevicePreset::IPhone14ProMax => DevicePreset::IPhone14ProMax,
            CliDevicePreset::IPhone14Pro => DevicePreset::IPhone14Pro,
            CliDevicePreset::IPhone14 => DevicePreset::IPhone14,
            CliDevicePreset::IPhoneSE => DevicePreset::IPhoneSE,
            CliDevicePreset::Desktop4K => DevicePreset::Desktop4K,
            CliDevicePreset::Desktop1440p => DevicePreset::Desktop1440p,
            CliDevicePreset::Desktop1080p => DevicePreset::Desktop1080p,
        }
    }
}

/// Command-line arguments for the bear flag generator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Device preset to generate wallpaper for
    #[arg(short, long, value_enum, default_value = "desktop-4k")]
    device: CliDevicePreset,

    /// Custom output path (overrides default based on dimensions)
    /// Format is auto-detected from extension if --format is not specified
    #[arg(short, long)]
    output: Option<String>,

    /// Output image format
    #[arg(long, value_enum)]
    format: Option<CliFormat>,

    /// Quality setting for lossy formats (0-100, default: 95)
    #[arg(long, default_value = "95")]
    quality: u8,

    /// Custom width in pixels (overrides device preset)
    #[arg(long)]
    width: Option<u32>,

    /// Custom height in pixels (overrides device preset)
    #[arg(long)]
    height: Option<u32>,

    /// Size of the bear paw as a fraction of flag height (0.01-1.0)
    #[arg(long, default_value = "0.35")]
    paw_size: f32,

    /// Place paw in bottom-left instead of center
    #[arg(long)]
    bottom_left: bool,

    /// Generate flags for multiple devices (comma-separated list)
    /// Example: --batch iphone-14-pro-max,iphone-14-pro,desktop-1080p
    #[arg(long)]
    batch: Option<String>,

    /// Generate flags for all available device presets
    #[arg(long)]
    batch_all: bool,

    /// Output directory for batch generation (default: current directory)
    #[arg(long)]
    output_dir: Option<PathBuf>,
}

fn parse_batch_list(batch_str: &str) -> Result<Vec<DevicePreset>, FlagError> {
    batch_str
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            let cli_preset = match trimmed.to_lowercase().as_str() {
                "iphone-14-pro-max" => CliDevicePreset::IPhone14ProMax,
                "iphone-14-pro" => CliDevicePreset::IPhone14Pro,
                "iphone-14" => CliDevicePreset::IPhone14,
                "iphone-se" => CliDevicePreset::IPhoneSE,
                "desktop-4k" => CliDevicePreset::Desktop4K,
                "desktop-1440p" => CliDevicePreset::Desktop1440p,
                "desktop-1080p" => CliDevicePreset::Desktop1080p,
                _ => {
                    return Err(FlagError::InvalidConfig(format!(
                        "Unknown device preset: {}. Use: iphone-14-pro-max, iphone-14-pro, iphone-14, iphone-se, desktop-4k, desktop-1440p, desktop-1080p",
                        trimmed
                    )));
                }
            };
            Ok(cli_preset.into())
        })
        .collect()
}

fn build_config(
    cli: &Cli,
    preset: Option<DevicePreset>,
) -> Result<FlagConfig, FlagError> {
    let mut config = if let (Some(width), Some(height)) = (cli.width, cli.height) {
        // Custom dimensions override device preset
        FlagConfig {
            width,
            height,
            output_path: format!("bear_flag_{}x{}.png", width, height),
            paw_size_ratio: cli.paw_size,
            center_paw: !cli.bottom_left,
            output_format: OutputFormat::Png,
            quality: cli.quality,
        }
    } else if let Some(p) = preset {
        FlagConfig::from_preset(p)
    } else {
        FlagConfig::from_preset(cli.device.into())
    };

    // Apply CLI overrides
    config.paw_size_ratio = cli.paw_size;
    config.center_paw = !cli.bottom_left;
    config.quality = cli.quality;

    // Determine output format
    if let Some(format) = cli.format {
        config.output_format = format.into();
    } else if let Some(ref output) = cli.output {
        // Try to detect from extension
        match OutputFormat::from_path(output) {
            Ok(fmt) => config.output_format = fmt,
            Err(e) => {
                eprintln!("Warning: Could not determine format from path: {}", e);
            }
        }
    }

    // Apply custom output path if provided
    if let Some(output) = &cli.output {
        config.output_path = output.clone();
    }

    // Ensure extension matches format
    config.ensure_extension();

    Ok(config)
}

fn generate_single_flag(config: &FlagConfig, verbose: bool) -> Result<(), FlagError> {
    if verbose {
        let device_name = if config.width == 3840 && config.height == 2160 {
            "Desktop 4K"
        } else {
            "Custom"
        };
        println!("Generating gay bear pride flag...");
        println!("  Device: {}", device_name);
        println!("  Dimensions: {}x{}", config.width, config.height);
        println!("  Format: {:?}", config.output_format);
        println!("  Output: {}", config.output_path);
        println!(
            "  Paw position: {}",
            if config.center_paw {
                "centered"
            } else {
                "bottom-left"
            }
        );
    }

    generate_flag(config)?;

    if verbose {
        println!("? Flag generated successfully!");
    }

    Ok(())
}

fn generate_batch(
    presets: Vec<DevicePreset>,
    base_cli: &Cli,
    output_dir: Option<&PathBuf>,
) -> Result<(), FlagError> {
    let count = presets.len();
    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} flags ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let results: Vec<(DevicePreset, Result<(), FlagError>)> = presets
        .into_par_iter()
        .map(|preset| {
            let mut config = build_config(base_cli, Some(preset))?;

            // Apply output directory if specified
            if let Some(dir) = output_dir {
                let filename = std::path::Path::new(&config.output_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("bear_flag.png");
                config.output_path = dir.join(filename).to_string_lossy().to_string();
            }

            let result = generate_flag(&config);
            pb.inc(1);
            Ok((preset, result))
        })
        .collect::<Result<_, FlagError>>()?;

    pb.finish_with_message("All flags generated!");

    let mut errors = Vec::new();
    for (preset, result) in results {
        if let Err(e) = result {
            errors.push((preset, e));
        }
    }

    if !errors.is_empty() {
        eprintln!("\nErrors occurred during batch generation:");
        for (preset, error) in &errors {
            eprintln!("  {}: {}", preset.display_name(), error);
        }
        return Err(FlagError::InvalidConfig(format!(
            "{} of {} flags failed to generate",
            errors.len(),
            count
        )));
    }

    println!("\n? Successfully generated {} flags", count);
    Ok(())
}

fn main() -> Result<(), FlagError> {
    let cli = Cli::parse();

    // Batch generation modes
    if cli.batch_all {
        let all_presets = DevicePreset::all().to_vec();
        return generate_batch(all_presets, &cli, cli.output_dir.as_ref());
    }

    if let Some(ref batch_str) = cli.batch {
        let presets = parse_batch_list(batch_str)?;
        if presets.is_empty() {
            return Err(FlagError::InvalidConfig(
                "Batch list cannot be empty".to_string(),
            ));
        }
        return generate_batch(presets, &cli, cli.output_dir.as_ref());
    }

    // Single flag generation
    let config = build_config(&cli, None)?;
    generate_single_flag(&config, true)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_batch_list() {
        let result = parse_batch_list("desktop-4k,desktop-1080p").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], DevicePreset::Desktop4K);
        assert_eq!(result[1], DevicePreset::Desktop1080p);
    }

    #[test]
    fn test_parse_batch_list_invalid() {
        assert!(parse_batch_list("invalid-preset").is_err());
    }

    #[test]
    fn test_parse_batch_list_with_spaces() {
        let result = parse_batch_list("desktop-4k, desktop-1080p , iphone-14").unwrap();
        assert_eq!(result.len(), 3);
    }
}
