use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use vibe_core::config::{ConfigWatcher, VibConfig};
use vibe_core::state::AppState;
use vibe_core::types::AnimationMode;

#[derive(Parser, Debug)]
#[command(name = "vibe")]
#[command(about = "Fluid Ambient Aura & Curved Contour Orbit for Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the background ambient overlay daemon
    Daemon,
    /// Open the rich matte-black settings GUI
    Settings,
    /// Toggle ambient glow & tracer on or off
    Toggle,
    /// Cycle to the next animation mode
    NextMode,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let state = AppState::new();

    // Load initial config
    let initial_config = VibConfig::load().unwrap_or_default();
    state.set_config(initial_config);

    match cli.command.unwrap_or(Commands::Daemon) {
        Commands::Daemon => run_daemon(state).await?,
        Commands::Settings => {
            log::info!("Launching vibe settings application...");

            // Start audio capture loopback for settings live preview
            let state_for_audio = state.clone();
            let _audio_capture = vibe_audio::AudioCapture::start(move |audio_state| {
                state_for_audio.set_audio(audio_state);
            });

            // Start MPRIS watcher for settings live preview
            let state_for_mpris = state.clone();
            tokio::spawn(async move {
                let watcher = vibe_mpris::MprisWatcher::new(state_for_mpris);
                let _ = watcher.run().await;
            });

            vibe_ui::run_settings_window(state)?;
        }
        Commands::Toggle => {
            let mut cfg = VibConfig::load().unwrap_or_default();
            if cfg.tracer_enabled || cfg.glow_intensity > 0.05 {
                cfg.tracer_enabled = false;
                cfg.glow_intensity = 0.0;
                log::info!("Toggled vibe state: OFF (glow_intensity=0.0, tracer=false)");
            } else {
                cfg.tracer_enabled = true;
                cfg.glow_intensity = 0.7;
                log::info!("Toggled vibe state: ON (glow_intensity=0.7, tracer=true)");
            }
            cfg.save()?;
        }
        Commands::NextMode => {
            let mut cfg = VibConfig::load().unwrap_or_default();
            cfg.animation_mode = match cfg.animation_mode {
                AnimationMode::OrbitTracer => AnimationMode::BeatPulse,
                AnimationMode::BeatPulse => AnimationMode::FluidWave,
                AnimationMode::FluidWave => AnimationMode::AmbientBreathe,
                AnimationMode::AmbientBreathe => AnimationMode::SteadyGlow,
                AnimationMode::SteadyGlow => AnimationMode::OrbitTracer,
            };
            cfg.save()?;
            log::info!("Switched animation mode to: {:?}", cfg.animation_mode);
        }
    }

    Ok(())
}

async fn run_daemon(state: AppState) -> Result<()> {
    log::info!("Starting vibe daemon (Wayland overlay + audio loopback + MPRIS)...");

    // 1. Hot-reload configuration file changes on save
    let state_for_watcher = state.clone();
    let _watcher = ConfigWatcher::start(move |new_config| {
        log::info!("Config hot-reloaded: thickness={}, intensity={}, tracer={}", 
            new_config.glow_thickness, new_config.glow_intensity, new_config.tracer_enabled);
        state_for_watcher.set_config(new_config);
    })?;

    // 2. Start PipeWire/ALSA low-latency audio capture
    let state_for_audio = state.clone();
    let _audio_capture = vibe_audio::AudioCapture::start(move |audio_state| {
        state_for_audio.set_audio(audio_state);
    })?;

    // 3. Start MPRIS2 D-Bus metadata watcher
    let state_for_mpris = state.clone();
    tokio::spawn(async move {
        let watcher = vibe_mpris::MprisWatcher::new(state_for_mpris);
        if let Err(e) = watcher.run().await {
            log::error!("MPRIS watcher exited: {:#}", e);
        }
    });

    // 4. Initialize Wayland OpenGL / EGL overlay
    let mut overlay = match vibe_render::WaylandOverlay::new(1920, 1080) {
        Ok(ov) => ov,
        Err(e) => {
            log::warn!("Could not connect to Wayland display server ({:#}). Running in headless audio mode.", e);
            // Run loop without rendering if headless
            while state.is_running() {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            return Ok(());
        }
    };

    log::info!("Vibe overlay active: 60 FPS GLSL shader pipeline running.");

    // 5. Main 60 FPS Render Loop
    while state.is_running() {
        let config = state.config();
        let audio = state.audio();
        let palette = state.palette();

        overlay.render_frame(&config, &audio, &palette);

        // Sleep ~16ms for 60 FPS target
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    Ok(())
}
