use std::path::Path;
use anyhow::{Context, Result};
use vibe_core::types::{Color, Palette};

/// Extract a dominant colour palette from an image file path or data buffer.
pub fn extract_palette_from_bytes(bytes: &[u8]) -> Result<Palette> {
    let img = image::load_from_memory(bytes).context("Failed to decode image data")?;
    let rgb_img = img.to_rgb8();
    let raw_pixels = rgb_img.as_raw();

    let palette_colors = color_thief::get_palette(raw_pixels, color_thief::ColorFormat::Rgb, 10, 5)
        .map_err(|e| anyhow::anyhow!("Color thief palette extraction error: {:?}", e))?;

    if palette_colors.is_empty() {
        return Ok(Palette::default());
    }

    let primary = Color::new(
        palette_colors[0].r as f32 / 255.0,
        palette_colors[0].g as f32 / 255.0,
        palette_colors[0].b as f32 / 255.0,
    );

    let secondary = if palette_colors.len() > 1 {
        Color::new(
            palette_colors[1].r as f32 / 255.0,
            palette_colors[1].g as f32 / 255.0,
            palette_colors[1].b as f32 / 255.0,
        )
    } else {
        Color::new(
            (primary.r * 0.7 + 0.2).clamp(0.0, 1.0),
            (primary.g * 0.5 + 0.3).clamp(0.0, 1.0),
            (primary.b * 0.9).clamp(0.0, 1.0),
        )
    };

    let accent = if palette_colors.len() > 2 {
        Color::new(
            palette_colors[2].r as f32 / 255.0,
            palette_colors[2].g as f32 / 255.0,
            palette_colors[2].b as f32 / 255.0,
        )
    } else {
        primary.lerp(secondary, 0.5)
    };

    Ok(Palette {
        primary,
        secondary,
        accent,
    })
}

/// Extract palette from a file path or URI (e.g. file:///path/to/cover.jpg or https://...).
pub fn extract_palette_from_uri(uri: &str) -> Result<Palette> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        // Fetch remote artwork with curl (standard on Linux)
        let output = std::process::Command::new("curl")
            .args(["-sL", "--max-time", "3", uri])
            .output()
            .with_context(|| format!("Failed to download remote cover art from {}", uri))?;

        if output.status.success() && !output.stdout.is_empty() {
            return extract_palette_from_bytes(&output.stdout);
        } else {
            anyhow::bail!("Failed to fetch cover art HTTP status: {:?}", output.status);
        }
    }

    let file_path = if let Some(stripped) = uri.strip_prefix("file://") {
        stripped
    } else {
        uri
    };

    // Decode URL-encoded characters (like %20 for space)
    let decoded_path = urlencoding_decode(file_path);
    let bytes = std::fs::read(Path::new(&decoded_path))
        .with_context(|| format!("Failed to read album art at {}", decoded_path))?;
    extract_palette_from_bytes(&bytes)
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(ch);
        }
    }
    result
}
