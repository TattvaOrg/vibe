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

/// Extract palette from a file path or URI (e.g. file:///path/to/cover.jpg).
pub fn extract_palette_from_uri(uri: &str) -> Result<Palette> {
    if let Some(file_path) = uri.strip_prefix("file://") {
        let bytes = std::fs::read(Path::new(file_path))
            .with_context(|| format!("Failed to read album art at {}", file_path))?;
        extract_palette_from_bytes(&bytes)
    } else if uri.starts_with('/') {
        let bytes = std::fs::read(Path::new(uri))
            .with_context(|| format!("Failed to read album art at {}", uri))?;
        extract_palette_from_bytes(&bytes)
    } else {
        anyhow::bail!("Remote HTTP cover downloads without network client not supported directly in offline mode")
    }
}
