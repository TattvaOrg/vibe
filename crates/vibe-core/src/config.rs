use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Result};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};

use crate::types::{AnimationMode, BeatSensitivity, ColorSource};

/// Top-level configuration persisted in `~/.config/vibe/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibConfig {
    /// Pixel width of the glow ring.
    #[serde(default = "default_glow_thickness")]
    pub glow_thickness: f32,

    /// How quickly the glow fades outward (0.0–1.0).
    #[serde(default = "default_glow_diffusion")]
    pub glow_diffusion: f32,

    /// Peak brightness of the glow (0.0–1.0).
    #[serde(default = "default_glow_intensity")]
    pub glow_intensity: f32,

    /// Whether the orbiting tracer is drawn.
    #[serde(default = "default_tracer_enabled")]
    pub tracer_enabled: bool,

    /// Tracer orbit speed multiplier.
    #[serde(default = "default_tracer_speed")]
    pub tracer_speed: f32,

    /// Length of the tracer tail relative to the perimeter (0.15–1.0).
    #[serde(default = "default_tracer_tail_length")]
    pub tracer_tail_length: f32,

    /// Beat-detection sensitivity preset.
    #[serde(default)]
    pub beat_sensitivity: BeatSensitivity,

    /// Active animation style.
    #[serde(default)]
    pub animation_mode: AnimationMode,

    /// Source for the colour palette.
    #[serde(default)]
    pub color_source: ColorSource,

    /// Primary glow colour (linear RGB).
    #[serde(default = "default_primary_color")]
    pub primary_color: [f32; 3],

    /// Secondary glow colour (linear RGB).
    #[serde(default = "default_secondary_color")]
    pub secondary_color: [f32; 3],

    /// Blend weight toward primary colour (0.0–1.0).
    #[serde(default = "default_color_weight")]
    pub color_weight: f32,

    /// Window corner radius in pixels.
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,

    /// Gap between the glow ring and the screen edge in pixels.
    #[serde(default = "default_screen_inset")]
    pub screen_inset: f32,

    /// Whether the now-playing overlay is visible.
    #[serde(default = "default_now_playing_visible")]
    pub now_playing_visible: bool,

    /// Whether the overlay auto-hides after inactivity.
    #[serde(default = "default_now_playing_auto_hide")]
    pub now_playing_auto_hide: bool,

    /// Seconds before auto-hiding the overlay.
    #[serde(default = "default_now_playing_auto_hide_secs")]
    pub now_playing_auto_hide_secs: u32,
}

// ── serde default helpers ────────────────────────────────────────────

fn default_glow_thickness() -> f32 { 40.0 }
fn default_glow_diffusion() -> f32 { 0.5 }
fn default_glow_intensity() -> f32 { 0.7 }
fn default_tracer_enabled() -> bool { true }
fn default_tracer_speed() -> f32 { 1.0 }
fn default_tracer_tail_length() -> f32 { 0.4 }
fn default_primary_color() -> [f32; 3] { [0.4, 0.2, 0.8] }
fn default_secondary_color() -> [f32; 3] { [0.2, 0.6, 0.9] }
fn default_color_weight() -> f32 { 0.6 }
fn default_corner_radius() -> f32 { 32.0 }
fn default_screen_inset() -> f32 { 4.0 }
fn default_now_playing_visible() -> bool { true }
fn default_now_playing_auto_hide() -> bool { true }
fn default_now_playing_auto_hide_secs() -> u32 { 5 }

impl Default for VibConfig {
    fn default() -> Self {
        Self {
            glow_thickness: default_glow_thickness(),
            glow_diffusion: default_glow_diffusion(),
            glow_intensity: default_glow_intensity(),
            tracer_enabled: default_tracer_enabled(),
            tracer_speed: default_tracer_speed(),
            tracer_tail_length: default_tracer_tail_length(),
            beat_sensitivity: BeatSensitivity::default(),
            animation_mode: AnimationMode::default(),
            color_source: ColorSource::default(),
            primary_color: default_primary_color(),
            secondary_color: default_secondary_color(),
            color_weight: default_color_weight(),
            corner_radius: default_corner_radius(),
            screen_inset: default_screen_inset(),
            now_playing_visible: default_now_playing_visible(),
            now_playing_auto_hide: default_now_playing_auto_hide(),
            now_playing_auto_hide_secs: default_now_playing_auto_hide_secs(),
        }
    }
}

impl VibConfig {
    /// Return the directory that holds the configuration file.
    fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("", "", "vibe")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let mut p = dirs_fallback_home();
                p.push(".config");
                p.push("vibe");
                p
            })
    }

    /// Full path to the configuration file.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load the configuration from `~/.config/vibe/config.toml`.
    ///
    /// If the file does not exist a default configuration is written and returned.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            let cfg = Self::default();
            cfg.save().context("writing default config")?;
            return Ok(cfg);
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let cfg: Self = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;

        Ok(cfg)
    }

    /// Persist the current configuration to `~/.config/vibe/config.toml`.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }

        let text = toml::to_string_pretty(self)
            .context("serializing config")?;

        fs::write(&path, text)
            .with_context(|| format!("writing {}", path.display()))?;

        log::info!("config saved to {}", path.display());
        Ok(())
    }
}

/// Fallback `$HOME` when `directories` cannot determine project dirs.
fn dirs_fallback_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

// ── Config file watcher ──────────────────────────────────────────────

/// Watches `~/.config/vibe/config.toml` for modifications and invokes a
/// callback with the freshly-loaded [`VibConfig`] whenever the file changes.
pub struct ConfigWatcher {
    /// Hold the watcher so it is not dropped (which would stop watching).
    _watcher: RecommendedWatcher,
    /// Handle to the background thread that dispatches callbacks.
    _thread: thread::JoinHandle<()>,
}

impl ConfigWatcher {
    /// Start watching the config file.
    ///
    /// `on_change` is called on a **background thread** every time the
    /// configuration file is modified on disk.
    pub fn start<F>(on_change: F) -> Result<Self>
    where
        F: Fn(VibConfig) + Send + 'static,
    {
        let config_path = VibConfig::config_path();
        let watch_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Ensure the directory exists so the watcher has something to watch.
        fs::create_dir_all(&watch_dir)
            .with_context(|| format!("creating config dir {}", watch_dir.display()))?;

        let (tx, rx) = mpsc::channel::<()>();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                // Only react to content modifications / creation.
                if matches!(
                    event.kind,
                    EventKind::Modify(notify::event::ModifyKind::Data(_))
                        | EventKind::Create(_)
                ) {
                    let _ = tx.send(());
                }
            }
        })
        .context("creating fs watcher")?;

        watcher
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}", watch_dir.display()))?;

        let config_path_clone = config_path.clone();
        let handle = thread::Builder::new()
            .name("config-watcher".into())
            .spawn(move || {
                for () in &rx {
                    // Coalesce rapid successive writes by draining the channel.
                    while rx.try_recv().is_ok() {}

                    match VibConfig::load() {
                        Ok(cfg) => {
                            log::info!("config reloaded from {}", config_path_clone.display());
                            on_change(cfg);
                        }
                        Err(e) => {
                            log::warn!("failed to reload config: {e:#}");
                        }
                    }
                }
            })
            .context("spawning config watcher thread")?;

        Ok(Self {
            _watcher: watcher,
            _thread: handle,
        })
    }
}
