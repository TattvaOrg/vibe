use std::sync::Arc;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use vibe_core::types::AudioState;

/// Real-time FFT analysis and beat detection engine.
pub struct AudioAnalyzer {
    fft_size: usize,
    sample_rate: f32,
    fft: Arc<dyn Fft<f32>>,
    rolling_energy: f32,
    rolling_bass: f32,
    beat_cooldown_frames: usize,
    last_beat_frame: usize,
    frame_counter: usize,
}

impl AudioAnalyzer {
    pub fn new(fft_size: usize, sample_rate: f32) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        Self {
            fft_size,
            sample_rate,
            fft,
            rolling_energy: 0.05,
            rolling_bass: 0.05,
            beat_cooldown_frames: 12,
            last_beat_frame: 0,
            frame_counter: 0,
        }
    }

    /// Process a slice of PCM audio samples and calculate the AudioState.
    pub fn process(&mut self, samples: &[f32]) -> AudioState {
        self.frame_counter += 1;

        if samples.is_empty() {
            return AudioState::default();
        }

        // Calculate basic RMS energy
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt().min(1.0);

        // Prepare FFT buffer
        let len = samples.len().min(self.fft_size);
        let mut buffer: Vec<Complex<f32>> = Vec::with_capacity(self.fft_size);

        // Apply Hann window
        for (i, &sample) in samples.iter().take(len).enumerate() {
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / len as f32).cos());
            buffer.push(Complex::new(sample * window, 0.0));
        }
        while buffer.len() < self.fft_size {
            buffer.push(Complex::new(0.0, 0.0));
        }

        // Run FFT
        self.fft.process(&mut buffer);

        // Compute magnitude spectrum for the positive frequencies
        let num_bins = self.fft_size / 2;
        let bin_width = self.sample_rate / self.fft_size as f32;

        let mut sub_bass_sum = 0.0f32;
        let mut bass_sum = 0.0f32;
        let mut mid_sum = 0.0f32;
        let mut high_sum = 0.0f32;

        let mut sub_bass_count = 0usize;
        let mut bass_count = 0usize;
        let mut mid_count = 0usize;
        let mut high_count = 0usize;

        for (bin_idx, complex) in buffer.iter().take(num_bins).enumerate() {
            let freq = bin_idx as f32 * bin_width;
            let mag = (complex.norm_sqr() / self.fft_size as f32).sqrt();

            if (20.0..60.0).contains(&freq) {
                sub_bass_sum += mag;
                sub_bass_count += 1;
            } else if (60.0..250.0).contains(&freq) {
                bass_sum += mag;
                bass_count += 1;
            } else if (250.0..4000.0).contains(&freq) {
                mid_sum += mag;
                mid_count += 1;
            } else if freq >= 4000.0 && freq < 16000.0 {
                high_sum += mag;
                high_count += 1;
            }
        }

        let sub_bass = if sub_bass_count > 0 { (sub_bass_sum / sub_bass_count as f32) * 5.0 } else { 0.0 };
        let bass = if bass_count > 0 { (bass_sum / bass_count as f32) * 4.0 } else { 0.0 };
        let mid = if mid_count > 0 { (mid_sum / mid_count as f32) * 3.0 } else { 0.0 };
        let high = if high_count > 0 { (high_sum / high_count as f32) * 2.0 } else { 0.0 };

        let combined_bass = (sub_bass * 0.6 + bass * 0.4).min(1.0);

        // Beat detection logic: detect peak differential against rolling bass average
        let alpha = 0.05f32;
        self.rolling_energy = (1.0 - alpha) * self.rolling_energy + alpha * rms;
        self.rolling_bass = (1.0 - alpha) * self.rolling_bass + alpha * combined_bass;

        let mut beat = false;
        let threshold = (self.rolling_bass * 1.4).max(0.12);

        if combined_bass > threshold && (self.frame_counter - self.last_beat_frame) > self.beat_cooldown_frames {
            beat = true;
            self.last_beat_frame = self.frame_counter;
        }

        AudioState {
            energy: rms.clamp(0.0, 1.0),
            beat,
            bass_energy: combined_bass.clamp(0.0, 1.0),
            mid_energy: mid.clamp(0.0, 1.0),
            high_energy: high.clamp(0.0, 1.0),
            bpm: 120.0, // Default baseline tempo
        }
    }
}
