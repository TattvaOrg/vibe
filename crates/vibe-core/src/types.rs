use serde::{Deserialize, Serialize};

/// Frequency-band weighting for beat detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeatSensitivity {
    Low,
    Balanced,
    Aggressive,
}

impl Default for BeatSensitivity {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Which animation drives the glow ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationMode {
    OrbitTracer,
    BeatPulse,
    FluidWave,
    AmbientBreathe,
    SteadyGlow,
}

impl Default for AnimationMode {
    fn default() -> Self {
        Self::OrbitTracer
    }
}

/// Visual texture / rendering style of the moving tracer line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TracerStyle {
    #[default]
    AuroraWave,
    NeonGlow,
    ElectricPulse,
}

/// Orbit direction along the screen perimeter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TracerDirection {
    #[default]
    Clockwise,
    CounterClockwise,
}

/// Where the palette colours come from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorSource {
    AlbumArt,
    ScreenEdge,
    Preset(String),
}

impl Default for ColorSource {
    fn default() -> Self {
        Self::AlbumArt
    }
}

/// Shared audio analysis snapshot consumed by the renderer.
#[derive(Debug, Clone)]
pub struct AudioState {
    /// Overall RMS energy in 0.0–1.0.
    pub energy: f32,
    /// True on the frame a beat is detected.
    pub beat: bool,
    /// Low-frequency band energy.
    pub bass_energy: f32,
    /// Mid-frequency band energy.
    pub mid_energy: f32,
    /// High-frequency band energy.
    pub high_energy: f32,
    /// Estimated beats-per-minute.
    pub bpm: f32,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            energy: 0.0,
            beat: false,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            bpm: 0.0,
        }
    }
}

/// Metadata about the currently-playing track.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// URI or file path to album artwork, if available.
    pub art_url: Option<String>,
    /// Whether playback is active.
    pub playing: bool,
    /// Current playback position in seconds.
    pub position: f64,
    /// Total track duration in seconds.
    pub duration: f64,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            art_url: None,
            playing: false,
            position: 0.0,
            duration: 0.0,
        }
    }
}

/// Linear-RGB colour (components in 0.0–1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    /// Create a new colour from RGB components.
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Linearly interpolate between `self` and `other` by factor `t` (0.0–1.0).
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }

    /// Parse a `#RRGGBB` hex string (the leading `#` is optional).
    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        anyhow::ensure!(hex.len() == 6, "Expected 6-character hex string, got {}", hex.len());
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        Ok(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        })
    }

    /// Return the colour as a three-element array.
    pub fn to_array(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0 }
    }
}

/// A colour palette derived from album art, screen edges, or a user preset.
#[derive(Debug, Clone)]
pub struct Palette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            primary: Color::new(0.4, 0.2, 0.8),
            secondary: Color::new(0.2, 0.6, 0.9),
            accent: Color::new(0.8, 0.3, 0.5),
        }
    }
}
