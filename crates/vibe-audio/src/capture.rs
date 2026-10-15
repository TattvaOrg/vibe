use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use vibe_core::types::AudioState;

use crate::analysis::AudioAnalyzer;

/// Background audio capturer monitoring system audio streams.
pub struct AudioCapture {
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl AudioCapture {
    /// Start audio capture in a background thread, reporting updates via the callback.
    pub fn start<F>(on_audio_state: F) -> Result<Self>
    where
        F: Fn(AudioState) + Send + Sync + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let thread_handle = thread::Builder::new()
            .name("vibe-audio-capture".into())
            .spawn(move || {
                let callback = Arc::new(on_audio_state);
                while running_clone.load(Ordering::Relaxed) {
                    if let Err(e) = Self::run_capture_loop(&running_clone, callback.clone()) {
                        log::warn!("Audio capture loop error, retrying in 2s: {:#}", e);
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            })
            .context("Spawning audio capture thread")?;

        Ok(Self {
            running,
            thread_handle: Some(thread_handle),
        })
    }

    fn run_capture_loop(
        running: &Arc<AtomicBool>,
        callback: Arc<dyn Fn(AudioState) + Send + Sync>,
    ) -> Result<()> {
        let host = cpal::default_host();

        // Attempt to find default input or monitor device
        let device = host
            .default_input_device()
            .context("No audio capture device found on host")?;

        log::info!("Audio capture using device: {}", device.name().unwrap_or_else(|_| "Default".into()));

        let default_config = device.default_input_config().context("Failed to get default input config")?;
        let sample_rate = default_config.sample_rate().0 as f32;
        let channels = default_config.channels() as usize;

        let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

        let err_fn = |err| log::error!("An error occurred on audio stream: {}", err);

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &default_config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = if channels > 1 {
                        data.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect()
                    } else {
                        data.to_vec()
                    };
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &default_config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = if channels > 1 {
                        data.chunks(channels)
                            .map(|c| c.iter().map(|&s| s as f32 / i16::MAX as f32).sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
                    };
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format for audio capture"),
        };

        stream.play().context("Failed to start audio stream")?;

        let mut analyzer = AudioAnalyzer::new(2048, sample_rate);
        let mut sample_accumulator: Vec<f32> = Vec::with_capacity(2048);

        while running.load(Ordering::Relaxed) {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(samples) => {
                    sample_accumulator.extend(samples);
                    if sample_accumulator.len() >= 2048 {
                        let state = analyzer.process(&sample_accumulator[..2048]);
                        callback(state);
                        sample_accumulator.drain(..1024); // 50% overlap for fluid tracking
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Send idle/quiet state on timeout
                    callback(AudioState::default());
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(())
    }

    /// Stop audio capture
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}
