use anyhow::Result;
use slint::ComponentHandle;
use vibe_core::state::AppState;
use vibe_core::types::{ColorSource, TracerDirection, TracerStyle};

slint::include_modules!();

/// Launch the Slint settings window.
pub fn run_settings_window(state: AppState) -> Result<()> {
    let window = SettingsWindow::new().map_err(|e| anyhow::anyhow!("Failed to create window: {:?}", e))?;

    let config = state.config();
    let media = state.media();

    // Initialize values from current config & media
    window.set_glow_thickness(config.glow_thickness);
    window.set_glow_diffusion(config.glow_diffusion);
    window.set_glow_intensity(config.glow_intensity);
    window.set_tracer_enabled(config.tracer_enabled);
    window.set_tracer_speed(config.tracer_speed);
    window.set_tracer_tail(config.tracer_tail_length);
    window.set_tracer_width(config.tracer_width);
    window.set_tracer_glow(config.tracer_glow);
    window.set_tracer_pause_enabled(config.tracer_pause_enabled);
    window.set_tracer_run_length(config.tracer_run_length);
    window.set_tracer_pause_duration_ms(config.tracer_pause_duration_ms as f32);
    window.set_tracer_pause_charge_up(config.tracer_pause_charge_up);
    window.set_tracer_pause_beat_breakout(config.tracer_pause_beat_breakout);

    window.set_tracer_direction(match config.tracer_direction {
        TracerDirection::Clockwise => 0,
        TracerDirection::CounterClockwise => 1,
    });

    window.set_tracer_style(match config.tracer_style {
        TracerStyle::AuroraWave => 0,
        TracerStyle::NeonGlow => 1,
        TracerStyle::ElectricPulse => 2,
    });

    window.set_active_color_idx(config.preset_color_index as i32);

    if !media.title.is_empty() {
        window.set_track_title(media.title.into());
        window.set_track_artist(media.artist.into());
    }

    // Dynamic timer to keep track metadata and audio status synchronized in real time
    let timer = slint::Timer::default();
    let state_for_timer = state.clone();
    let window_handle_timer = window.as_weak();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(200), move || {
        if let Some(w) = window_handle_timer.upgrade() {
            let m = state_for_timer.media();
            let a = state_for_timer.audio();

            if m.playing {
                w.set_track_title(m.title.into());
                w.set_track_artist(if m.artist.is_empty() { "Media Player".into() } else { m.artist.into() });
            } else if !m.title.is_empty() {
                w.set_track_title(format!("[Paused] {}", m.title).into());
                w.set_track_artist(if m.artist.is_empty() { "Media Player".into() } else { m.artist.into() });
            } else if a.energy > 0.02 {
                w.set_track_title("Active Audio Stream".into());
                w.set_track_artist("PipeWire Audio Capture".into());
            } else {
                w.set_track_title("No track playing".into());
                w.set_track_artist("Offline".into());
            }
        }
    });

    // Direction changed callback
    let state_clone_dir = state.clone();
    window.on_direction_changed(move |dir| {
        let mut cfg = state_clone_dir.config();
        cfg.tracer_direction = if dir == 1 {
            TracerDirection::CounterClockwise
        } else {
            TracerDirection::Clockwise
        };
        state_clone_dir.set_config(cfg.clone());
        let _ = cfg.save();
    });

    // Style changed callback
    let state_clone_style = state.clone();
    window.on_style_changed(move |style| {
        let mut cfg = state_clone_style.config();
        cfg.tracer_style = match style {
            1 => TracerStyle::NeonGlow,
            2 => TracerStyle::ElectricPulse,
            _ => TracerStyle::AuroraWave,
        };
        state_clone_style.set_config(cfg.clone());
        let _ = cfg.save();
    });

    // Color swatch selected callback
    let state_clone_color = state.clone();
    window.on_color_selected(move |idx| {
        let mut cfg = state_clone_color.config();
        cfg.preset_color_index = idx as u32;
        match idx {
            0 => {
                cfg.color_source = ColorSource::AlbumArt;
            }
            1 => {
                cfg.color_source = ColorSource::Preset("Arctic Emerald".to_string());
                cfg.primary_color = [0.0, 0.96, 0.63];
                cfg.secondary_color = [0.0, 0.65, 0.85];
            }
            2 => {
                cfg.color_source = ColorSource::Preset("Cyan Plasma".to_string());
                cfg.primary_color = [0.0, 0.83, 1.0];
                cfg.secondary_color = [0.2, 0.4, 0.95];
            }
            3 => {
                cfg.color_source = ColorSource::Preset("Cyber Violet".to_string());
                cfg.primary_color = [0.66, 0.33, 0.97];
                cfg.secondary_color = [0.93, 0.28, 0.6];
            }
            4 => {
                cfg.color_source = ColorSource::Preset("Electric Blue".to_string());
                cfg.primary_color = [0.23, 0.51, 0.96];
                cfg.secondary_color = [0.0, 0.85, 0.9];
            }
            5 => {
                cfg.color_source = ColorSource::Preset("Sunset Coral".to_string());
                cfg.primary_color = [1.0, 0.32, 0.32];
                cfg.secondary_color = [1.0, 0.65, 0.2];
            }
            6 => {
                cfg.color_source = ColorSource::Preset("Pure White".to_string());
                cfg.primary_color = [1.0, 1.0, 1.0];
                cfg.secondary_color = [0.8, 0.85, 0.9];
            }
            _ => {}
        }
        state_clone_color.set_config(cfg.clone());
        let _ = cfg.save();
    });

    // Slider adjustments callback
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_config_changed(move || {
        if let Some(w) = window_handle.upgrade() {
            let mut cfg = state_clone.config();
            cfg.glow_thickness = w.get_glow_thickness();
            cfg.glow_diffusion = w.get_glow_diffusion();
            cfg.glow_intensity = w.get_glow_intensity();
            cfg.tracer_enabled = w.get_tracer_enabled();
            cfg.tracer_speed = w.get_tracer_speed();
            cfg.tracer_tail_length = w.get_tracer_tail();
            cfg.tracer_width = w.get_tracer_width();
            cfg.tracer_glow = w.get_tracer_glow();
            cfg.tracer_pause_enabled = w.get_tracer_pause_enabled();
            cfg.tracer_run_length = w.get_tracer_run_length();
            cfg.tracer_pause_duration_ms = w.get_tracer_pause_duration_ms() as u32;
            cfg.tracer_pause_charge_up = w.get_tracer_pause_charge_up();
            cfg.tracer_pause_beat_breakout = w.get_tracer_pause_beat_breakout();

            state_clone.set_config(cfg.clone());
            let _ = cfg.save();
        }
    });

    // Close window callback
    let window_handle_close = window.as_weak();
    window.on_close_window(move || {
        if let Some(w) = window_handle_close.upgrade() {
            let _ = w.hide();
        }
    });

    window.run().map_err(|e| anyhow::anyhow!("Slint run error: {:?}", e))?;
    Ok(())
}
