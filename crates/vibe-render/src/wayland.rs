use std::sync::Arc;
use anyhow::{Context, Result};
use khronos_egl as egl;
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
    LayerSurfaceConfigure,
};
use smithay_client_toolkit::shell::WaylandSurface;
use vibe_core::config::VibConfig;
use vibe_core::types::{AudioState, Palette};
use wayland_client::protocol::{wl_output, wl_region, wl_surface};
use wayland_client::{Connection, Proxy, QueueHandle};
use wayland_egl::WlEglSurface;

use crate::renderer::OverlayRenderer;

/// Internal event state for Smithay Client Toolkit.
pub struct VibeWaylandState {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub exit: bool,
    pub width: u32,
    pub height: u32,
}

impl ProvidesRegistryState for VibeWaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    smithay_client_toolkit::registry_handlers![];
}

impl OutputHandler for VibeWaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl CompositorHandler for VibeWaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_scale: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for VibeWaylandState {
    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
    }
}

impl wayland_client::Dispatch<wl_region::WlRegion, ()> for VibeWaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_region::WlRegion,
        _event: wl_region::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

smithay_client_toolkit::delegate_compositor!(VibeWaylandState);
smithay_client_toolkit::delegate_output!(VibeWaylandState);
smithay_client_toolkit::delegate_layer!(VibeWaylandState);
smithay_client_toolkit::delegate_registry!(VibeWaylandState);

/// Dynamically loaded libwayland-client functions to avoid link-time errors.
struct WaylandClientDynamic {
    _lib: libloading::Library,
    disconnect: unsafe extern "C" fn(*mut std::ffi::c_void),
    display_ptr: *mut std::ffi::c_void,
}

impl WaylandClientDynamic {
    fn try_connect() -> Option<Self> {
        let lib_names = ["libwayland-client.so.0", "libwayland-client.so"];
        for name in lib_names {
            if let Ok(lib) = unsafe { libloading::Library::new(name) } {
                let connect_fn: Result<
                    libloading::Symbol<unsafe extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_void>,
                    _,
                > = unsafe { lib.get(b"wl_display_connect\0") };

                let disconnect_fn: Result<
                    libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)>,
                    _,
                > = unsafe { lib.get(b"wl_display_disconnect\0") };

                if let (Ok(connect), Ok(disconnect)) = (connect_fn, disconnect_fn) {
                    let ptr = unsafe { connect(std::ptr::null()) };
                    if !ptr.is_null() {
                        let disconnect_fn_ptr = *disconnect;
                        return Some(Self {
                            _lib: lib,
                            disconnect: disconnect_fn_ptr,
                            display_ptr: ptr,
                        });
                    }
                }
            }
        }
        None
    }
}

impl Drop for WaylandClientDynamic {
    fn drop(&mut self) {
        if !self.display_ptr.is_null() {
            unsafe {
                (self.disconnect)(self.display_ptr);
            }
        }
    }
}

/// Manages the Wayland display, layer-shell surface, and EGL context.
pub struct WaylandOverlay {
    egl_instance: Arc<egl::DynamicInstance<egl::EGL1_4>>,
    egl_display: egl::Display,
    egl_context: egl::Context,
    egl_surface: egl::Surface,
    _dynamic_client: Option<WaylandClientDynamic>,
    _wl_egl_surface: Option<WlEglSurface>,
    _layer_surface: Option<LayerSurface>,
    event_queue: Option<wayland_client::EventQueue<VibeWaylandState>>,
    wayland_state: Option<VibeWaylandState>,
    renderer: OverlayRenderer,
    width: u32,
    height: u32,
}

impl WaylandOverlay {
    /// Initialize a standalone or layer-shell connected EGL/OpenGL rendering surface.
    pub fn new(mut width: u32, mut height: u32) -> Result<Self> {
        let egl_instance = unsafe {
            egl::DynamicInstance::<egl::EGL1_4>::load_required()
                .map_err(|e| anyhow::anyhow!("Failed to load EGL library: {:?}", e))?
        };
        let egl_instance = Arc::new(egl_instance);

        // Dynamically connect to Wayland display socket if available
        let dynamic_client = WaylandClientDynamic::try_connect();

        let egl_display = if let Some(ref client) = dynamic_client {
            unsafe { egl_instance.get_display(client.display_ptr as *mut _) }
                .context("Failed to get EGL display for Wayland")?
        } else {
            unsafe { egl_instance.get_display(egl::DEFAULT_DISPLAY) }
                .context("Failed to get default EGL display")?
        };

        let (major, minor) = egl_instance
            .initialize(egl_display)
            .map_err(|e| anyhow::anyhow!("Failed to initialize EGL: {:?}", e))?;

        log::info!("Initialized EGL version {}.{}", major, minor);

        let config_attribs = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT | egl::PBUFFER_BIT,
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

        // Try creating Wayland Layer Shell surface via high-level Connection
        let mut created_layer = None;
        let mut created_wl_egl = None;
        let mut created_egl_surface = None;
        let mut created_queue = None;
        let mut created_state = None;

        if let Ok(conn) = Connection::connect_to_env() {
            let (globals, mut event_queue) =
                wayland_client::globals::registry_queue_init::<VibeWaylandState>(&conn)
                    .map_err(|e| anyhow::anyhow!("Wayland registry init failed: {:?}", e))?;

            let qh = event_queue.handle();
            let compositor_res = CompositorState::bind(&globals, &qh);
            let layer_shell_res = LayerShell::bind(&globals, &qh);

            if let (Ok(compositor), Ok(layer_shell)) = (compositor_res, layer_shell_res) {
                let surface = compositor.create_surface(&qh);
                let layer_surface = layer_shell.create_layer_surface(
                    &qh,
                    surface,
                    Layer::Overlay,
                    Some("vibe-overlay"),
                    None,
                );

                layer_surface
                    .set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
                layer_surface.set_exclusive_zone(-1);
                layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

                // Transparent click-through input region
                let region = compositor.wl_compositor().create_region(&qh, ());
                layer_surface.wl_surface().set_input_region(Some(&region));
                layer_surface.commit();

                let mut state = VibeWaylandState {
                    registry_state: RegistryState::new(&globals),
                    output_state: OutputState::new(&globals, &qh),
                    exit: false,
                    width,
                    height,
                };

                // Perform initial dispatch to receive compositor configure geometry
                let _ = event_queue.roundtrip(&mut state);
                if state.width > 0 && state.height > 0 {
                    width = state.width;
                    height = state.height;
                }

                let wl_surface_id = layer_surface.wl_surface().id();
                let wl_egl_res = WlEglSurface::new(wl_surface_id, width as i32, height as i32);

                if let Ok(wl_egl) = wl_egl_res {
                    let win_surface_res = unsafe {
                        egl_instance.create_window_surface(
                            egl_display,
                            config,
                            wl_egl.ptr() as *mut _,
                            None,
                        )
                    };

                    if let Ok(surf) = win_surface_res {
                        log::info!(
                            "Created native Wayland layer-shell EGL window surface ({}x{})",
                            width,
                            height
                        );
                        created_layer = Some(layer_surface);
                        created_wl_egl = Some(wl_egl);
                        created_egl_surface = Some(surf);
                        created_queue = Some(event_queue);
                        created_state = Some(state);
                    }
                }
            }
        }

        // Fallback pbuffer surface if layer-shell wasn't available
        let egl_surface = if let Some(surf) = created_egl_surface {
            surf
        } else {
            let pbuffer_attribs = [
                egl::WIDTH,
                width as i32,
                egl::HEIGHT,
                height as i32,
                egl::NONE,
            ];

            egl_instance
                .create_pbuffer_surface(egl_display, config, &pbuffer_attribs)
                .map_err(|e| anyhow::anyhow!("Failed to create EGL pbuffer surface: {:?}", e))?
        };

        egl_instance
            .make_current(
                egl_display,
                Some(egl_surface),
                Some(egl_surface),
                Some(egl_context),
            )
            .map_err(|e| anyhow::anyhow!("Failed to make EGL context current: {:?}", e))?;

        let gl = unsafe {
            let instance_clone = egl_instance.clone();
            glow::Context::from_loader_function(move |s| {
                instance_clone
                    .get_proc_address(s)
                    .map(|p| p as *const _)
                    .unwrap_or(std::ptr::null())
            })
        };

        log::info!("Initialized OpenGL 3.3 Core shader pipeline");

        let renderer = OverlayRenderer::new(gl)?;

        Ok(Self {
            egl_instance,
            egl_display,
            egl_context,
            egl_surface,
            _dynamic_client: dynamic_client,
            _wl_egl_surface: created_wl_egl,
            _layer_surface: created_layer,
            event_queue: created_queue,
            wayland_state: created_state,
            renderer,
            width,
            height,
        })
    }

    /// Render one frame with current system state.
    pub fn render_frame(&mut self, config: &VibConfig, audio: &AudioState, palette: &Palette) {
        let mut new_dims = None;
        if let (Some(queue), Some(state)) = (&mut self.event_queue, &mut self.wayland_state) {
            let _ = queue.dispatch_pending(state);
            if state.width != self.width || state.height != self.height {
                new_dims = Some((state.width, state.height));
            }
        }
        if let Some((w, h)) = new_dims {
            self.resize(w, h);
        }

        self.renderer.render(config, audio, palette, self.width, self.height);
        let _ = self.egl_instance.swap_buffers(self.egl_display, self.egl_surface);
    }

    /// Resize viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let Some(ref mut wl_egl) = self._wl_egl_surface {
            wl_egl.resize(width as i32, height as i32, 0, 0);
        }
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
