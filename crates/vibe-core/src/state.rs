use std::sync::{Arc, Mutex};

use crate::config::VibConfig;
use crate::types::{AudioState, MediaInfo, Palette};

/// Thread-safe shared application state.
///
/// Every field is wrapped in its own [`Mutex`] so that independent
/// subsystems (audio, MPRIS, renderer) can update their slice of state
/// without blocking each other.
#[derive(Debug, Clone)]
pub struct AppState {
    config: Arc<Mutex<VibConfig>>,
    audio: Arc<Mutex<AudioState>>,
    media: Arc<Mutex<MediaInfo>>,
    palette: Arc<Mutex<Palette>>,
    running: Arc<Mutex<bool>>,
}

impl AppState {
    /// Create a new `AppState` with default values.
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(VibConfig::default())),
            audio: Arc::new(Mutex::new(AudioState::default())),
            media: Arc::new(Mutex::new(MediaInfo::default())),
            palette: Arc::new(Mutex::new(Palette::default())),
            running: Arc::new(Mutex::new(true)),
        }
    }

    // ── Config ───────────────────────────────────────────────────────

    /// Return a snapshot of the current configuration.
    pub fn config(&self) -> VibConfig {
        self.config.lock().expect("config lock poisoned").clone()
    }

    /// Replace the current configuration.
    pub fn set_config(&self, cfg: VibConfig) {
        *self.config.lock().expect("config lock poisoned") = cfg;
    }

    // ── Audio ────────────────────────────────────────────────────────

    /// Return a snapshot of the current audio state.
    pub fn audio(&self) -> AudioState {
        self.audio.lock().expect("audio lock poisoned").clone()
    }

    /// Replace the current audio state.
    pub fn set_audio(&self, state: AudioState) {
        *self.audio.lock().expect("audio lock poisoned") = state;
    }

    // ── Media ────────────────────────────────────────────────────────

    /// Return a snapshot of the current media info.
    pub fn media(&self) -> MediaInfo {
        self.media.lock().expect("media lock poisoned").clone()
    }

    /// Replace the current media info.
    pub fn set_media(&self, info: MediaInfo) {
        *self.media.lock().expect("media lock poisoned") = info;
    }

    // ── Palette ──────────────────────────────────────────────────────

    /// Return a snapshot of the current palette.
    pub fn palette(&self) -> Palette {
        self.palette.lock().expect("palette lock poisoned").clone()
    }

    /// Replace the current palette.
    pub fn set_palette(&self, palette: Palette) {
        *self.palette.lock().expect("palette lock poisoned") = palette;
    }

    // ── Running flag ─────────────────────────────────────────────────

    /// Check whether the application should keep running.
    pub fn is_running(&self) -> bool {
        *self.running.lock().expect("running lock poisoned")
    }

    /// Signal the application to shut down.
    pub fn request_shutdown(&self) {
        *self.running.lock().expect("running lock poisoned") = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
