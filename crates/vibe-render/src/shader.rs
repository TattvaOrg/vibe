use anyhow::Result;
use glow::HasContext;

pub const VERTEX_SHADER_SRC: &str = include_str!("../../../assets/shaders/aura.vert");
pub const FRAGMENT_SHADER_SRC: &str = include_str!("../../../assets/shaders/aura.frag");

/// Compile a GLSL shader.
pub unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader> {
    unsafe {
        let shader = gl.create_shader(shader_type).map_err(|e| anyhow::anyhow!("Create shader error: {}", e))?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            anyhow::bail!("Shader compilation failed: {}", log);
        }

        Ok(shader)
    }
}

/// Link a vertex and fragment shader into a shader program.
pub unsafe fn link_program(
    gl: &glow::Context,
    vertex_shader: glow::Shader,
    fragment_shader: glow::Shader,
) -> Result<glow::Program> {
    unsafe {
        let program = gl.create_program().map_err(|e| anyhow::anyhow!("Create program error: {}", e))?;
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            anyhow::bail!("Program linking failed: {}", log);
        }

        Ok(program)
    }
}
