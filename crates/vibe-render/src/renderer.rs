use std::time::Instant;

use anyhow::{Context, Result};
use glow::HasContext;
use vibe_core::config::VibConfig;
use vibe_core::types::{AnimationMode, AudioState, Palette};

use crate::shader::{compile_shader, link_program, FRAGMENT_SHADER_SRC, VERTEX_SHADER_SRC};

/// Fullscreen GPU overlay renderer using OpenGL / GLSL.
pub struct OverlayRenderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    tracer_position: f32,
    last_frame: Instant,
    start_time: Instant,
}

impl OverlayRenderer {
    pub fn new(gl: glow::Context) -> Result<Self> {
        unsafe {
            let vs = compile_shader(&gl, glow::VERTEX_SHADER, VERTEX_SHADER_SRC)
                .context("Compiling aura vertex shader")?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SRC)
                .context("Compiling aura fragment shader")?;
            let program = link_program(&gl, vs, fs).context("Linking aura shader program")?;

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            // Fullscreen quad vertices (-1.0 to 1.0)
            let quad_vertices: [f32; 12] = [
                -1.0, -1.0,
                 1.0, -1.0,
                -1.0,  1.0,
                -1.0,  1.0,
                 1.0, -1.0,
                 1.0,  1.0,
            ];

            let vao = gl.create_vertex_array().map_err(|e| anyhow::anyhow!("{}", e))?;
            let vbo = gl.create_buffer().map_err(|e| anyhow::anyhow!("{}", e))?;

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck_cast_slice(&quad_vertices),
                glow::STATIC_DRAW,
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * std::mem::size_of::<f32>() as i32, 0);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            // Enable alpha blending for transparent overlay
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            Ok(Self {
                gl,
                program,
                vao,
                _vbo: vbo,
                tracer_position: 0.0,
                last_frame: Instant::now(),
                start_time: Instant::now(),
            })
        }
    }

    /// Render one frame of the ambient aura and tracer beam.
    pub fn render(
        &mut self,
        config: &VibConfig,
        audio: &AudioState,
        palette: &Palette,
        width: u32,
        height: u32,
    ) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        let elapsed = now.duration_since(self.start_time).as_secs_f32();

        // Advance white tracer beam along the perimeter
        // Acceleration kicks in when bass transients trigger
        let beat_multiplier = 1.0 + (audio.bass_energy * 1.5);
        let speed = (config.tracer_speed * 0.25) * beat_multiplier;
        self.tracer_position = (self.tracer_position + speed * dt).fract();

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.program));

            // Set uniforms
            self.set_uniform_2f("u_resolution", width as f32, height as f32);
            self.set_uniform_1f("u_time", elapsed);
            self.set_uniform_1f("u_thickness", config.glow_thickness);
            self.set_uniform_1f("u_diffusion", config.glow_diffusion);
            self.set_uniform_1f("u_intensity", config.glow_intensity);

            let primary = palette.primary.to_array();
            let secondary = palette.secondary.to_array();
            self.set_uniform_3f("u_primary_color", primary[0], primary[1], primary[2]);
            self.set_uniform_3f("u_secondary_color", secondary[0], secondary[1], secondary[2]);
            self.set_uniform_1f("u_color_weight", config.color_weight);
            self.set_uniform_1f("u_corner_radius", config.corner_radius);
            self.set_uniform_1f("u_screen_inset", config.screen_inset);

            self.set_uniform_1i("u_tracer_enabled", if config.tracer_enabled { 1 } else { 0 });
            self.set_uniform_1f("u_tracer_position", self.tracer_position);
            self.set_uniform_1f("u_tracer_tail", config.tracer_tail_length);
            self.set_uniform_1f("u_beat_energy", if audio.beat { 1.0 } else { audio.bass_energy });

            let mode_val = match config.animation_mode {
                AnimationMode::OrbitTracer => 0,
                AnimationMode::BeatPulse => 1,
                AnimationMode::FluidWave => 2,
                AnimationMode::AmbientBreathe => 3,
                AnimationMode::SteadyGlow => 4,
            };
            self.set_uniform_1i("u_mode", mode_val);

            // Draw quad
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }
    }

    unsafe fn set_uniform_1f(&self, name: &str, v: f32) {
        unsafe {
            if let Some(loc) = self.gl.get_uniform_location(self.program, name) {
                self.gl.uniform_1_f32(Some(&loc), v);
            }
        }
    }

    unsafe fn set_uniform_2f(&self, name: &str, v0: f32, v1: f32) {
        unsafe {
            if let Some(loc) = self.gl.get_uniform_location(self.program, name) {
                self.gl.uniform_2_f32(Some(&loc), v0, v1);
            }
        }
    }

    unsafe fn set_uniform_3f(&self, name: &str, v0: f32, v1: f32, v2: f32) {
        unsafe {
            if let Some(loc) = self.gl.get_uniform_location(self.program, name) {
                self.gl.uniform_3_f32(Some(&loc), v0, v1, v2);
            }
        }
    }

    unsafe fn set_uniform_1i(&self, name: &str, v: i32) {
        unsafe {
            if let Some(loc) = self.gl.get_uniform_location(self.program, name) {
                self.gl.uniform_1_i32(Some(&loc), v);
            }
        }
    }
}

impl Drop for OverlayRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self._vbo);
        }
    }
}

fn bytemuck_cast_slice(slice: &[f32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * std::mem::size_of::<f32>(),
        )
    }
}
