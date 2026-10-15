use anyhow::Result;
use slint::ComponentHandle;
use vibe_core::state::AppState;

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

    if !media.title.is_empty() {
        window.set_track_title(media.title.into());
        window.set_track_artist(media.artist.into());
    }

    let state_clone = state.clone();
    let window_handle = window.as_weak();

    // Callback on slider adjustments
    window.on_config_changed(move || {
        if let Some(w) = window_handle.upgrade() {
            let mut cfg = state_clone.config();
            cfg.glow_thickness = w.get_glow_thickness();
            cfg.glow_diffusion = w.get_glow_diffusion();
            cfg.glow_intensity = w.get_glow_intensity();
            cfg.tracer_enabled = w.get_tracer_enabled();
            cfg.tracer_speed = w.get_tracer_speed();
            cfg.tracer_tail_length = w.get_tracer_tail();

            state_clone.set_config(cfg.clone());
            let _ = cfg.save();
        }
    });

    let window_handle_close = window.as_weak();
    window.on_close_window(move || {
        if let Some(w) = window_handle_close.upgrade() {
            let _ = w.hide();
        }
    });

    window.run().map_err(|e| anyhow::anyhow!("Slint run error: {:?}", e))?;
    Ok(())
}
