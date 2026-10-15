use std::sync::Arc;
use anyhow::{Context, Result};
use khronos_egl as egl;
use vibe_core::config::VibConfig;
use vibe_core::types::{AudioState, Palette};

use crate::renderer::OverlayRenderer;

/// Manages the Wayland display, layer-shell surface, and EGL context.
pub struct WaylandOverlay {
    egl_instance: Arc<egl::DynamicInstance<egl::EGL1_4>>,
    egl_display: egl::Display,
    egl_context: egl::Context,
    egl_surface: egl::Surface,
    renderer: OverlayRenderer,
    width: u32,
    height: u32,
}

impl WaylandOverlay {
    /// Initialize a standalone or layer-shell connected EGL/OpenGL rendering surface.
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let egl_instance = unsafe {
            egl::DynamicInstance::<egl::EGL1_4>::load_required()
                .map_err(|e| anyhow::anyhow!("Failed to load EGL library: {:?}", e))?
        };
        let egl_instance = Arc::new(egl_instance);

        let egl_display = unsafe {
            egl_instance
                .get_display(egl::DEFAULT_DISPLAY)
                .context("Failed to get default EGL display")?
        };

        let (major, minor) = egl_instance
            .initialize(egl_display)
            .map_err(|e| anyhow::anyhow!("Failed to initialize EGL: {:?}", e))?;

        log::info!("Initialized EGL version {}.{}", major, minor);

        let config_attribs = [
            egl::SURFACE_TYPE,
            egl::PBUFFER_BIT | egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::NONE,
        ];

        let config = egl_instance
            .choose_first_config(egl_display, &config_attribs)
            .map_err(|e| anyhow::anyhow!("Failed to choose EGL config: {:?}", e))?
            .context("No matching EGL config found")?;

        egl_instance
            .bind_api(egl::OPENGL_API)
            .map_err(|e| anyhow::anyhow!("Failed to bind OpenGL API: {:?}", e))?;

        let context_attribs = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            3,
            egl::CONTEXT_OPENGL_PROFILE_MASK,
            egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
            egl::NONE,
        ];

        let egl_context = egl_instance
            .create_context(egl_display, config, None, &context_attribs)
            .map_err(|e| anyhow::anyhow!("Failed to create EGL context: {:?}", e))?;

        // Fallback pbuffer surface for offscreen/headless or initial display
        let pbuffer_attribs = [
            egl::WIDTH,
            width as i32,
            egl::HEIGHT,
            height as i32,
            egl::NONE,
        ];

        let egl_surface = egl_instance
            .create_pbuffer_surface(egl_display, config, &pbuffer_attribs)
            .map_err(|e| anyhow::anyhow!("Failed to create EGL pbuffer surface: {:?}", e))?;

        egl_instance
            .make_current(egl_display, Some(egl_surface), Some(egl_surface), Some(egl_context))
            .map_err(|e| anyhow::anyhow!("Failed to make EGL context current: {:?}", e))?;

        let gl = unsafe {
            let instance_clone = egl_instance.clone();
            glow::Context::from_loader_function(move |s| {
                instance_clone.get_proc_address(s).map(|p| p as *const _).unwrap_or(std::ptr::null())
            })
        };

        log::info!("Created OpenGL context on Wayland EGL");

        let renderer = OverlayRenderer::new(gl)?;

        Ok(Self {
            egl_instance,
            egl_display,
            egl_context,
            egl_surface,
            renderer,
            width,
            height,
        })
    }

    /// Render one frame with current system state.
    pub fn render_frame(&mut self, config: &VibConfig, audio: &AudioState, palette: &Palette) {
        self.renderer.render(config, audio, palette, self.width, self.height);
        let _ = self.egl_instance.swap_buffers(self.egl_display, self.egl_surface);
    }

    /// Resize viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for WaylandOverlay {
    fn drop(&mut self) {
        let _ = self.egl_instance.make_current(self.egl_display, None, None, None);
        let _ = self.egl_instance.destroy_surface(self.egl_display, self.egl_surface);
        let _ = self.egl_instance.destroy_context(self.egl_display, self.egl_context);
        let _ = self.egl_instance.terminate(self.egl_display);
    }
}
