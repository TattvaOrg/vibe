//! Color palette extracted from media artwork.

use serde::{Deserialize, Serialize};

/// RGB color with components in the 0.0–1.0 range (linear sRGB).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// A sensible default purple-ish accent.
    pub const DEFAULT_PRIMARY: Self = Self::new(0.54, 0.17, 0.89);
    /// A sensible default teal accent.
    pub const DEFAULT_SECONDARY: Self = Self::new(0.0, 0.78, 0.82);
}

/// A two-color palette driving the ambient aura.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Palette {
    pub primary: Color,
    pub secondary: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            primary: Color::DEFAULT_PRIMARY,
            secondary: Color::DEFAULT_SECONDARY,
        }
    }
}
