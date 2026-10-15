//! Real-time audio analysis state shared between the audio engine and renderer.

/// Snapshot of the current audio analysis state.
///
/// Updated at 60–120 Hz by the audio processing thread and read by the
/// renderer each frame.
#[derive(Debug, Clone, Copy)]
pub struct AudioState {
    /// Instantaneous beat energy in 0.0–1.0 (smoothed transient envelope).
    pub beat_energy: f32,
    /// Estimated tempo in beats per minute (0 when unknown).
    pub bpm: f32,
    /// Whether audio is currently playing (non-silent).
    pub is_active: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            beat_energy: 0.0,
            bpm: 0.0,
            is_active: false,
        }
    }
}
