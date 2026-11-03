//! A linked program and its uniform locations, looked up once by name.

use std::collections::HashMap;

use glow::HasContext as _;

use crate::Value;

/// A linked program; uniforms the program lacks are skipped.
pub(crate) struct Program {
    pub(crate) id: glow::Program,
    locations: HashMap<&'static str, Option<glow::UniformLocation>>,
}

impl Program {
    /// Compile and link `vertex` and `fragment`.
    pub(crate) fn new(gl: &glow::Context, vertex: &str, fragment: &str) -> Result<Self, String> {
        // SAFETY: on the caller's current context; the program is returned or deleted.
        let id = unsafe { link(gl, vertex, fragment)? };
        Ok(Self {
            id,
            locations: HashMap::new(),
        })
    }

    /// Set the uniform `name` on this program, which must be in use.
    pub(crate) fn set(&mut self, gl: &glow::Context, name: &'static str, value: &Value) {
        let program = self.id;
        let location = self
            .locations
            .entry(name)
            // SAFETY: a query on this linked program.
            .or_insert_with(|| unsafe { gl.get_uniform_location(program, name) })
            .as_ref();
        if location.is_none() {
            return;
        }
        // SAFETY: the location belongs to this program, which is in use.
        unsafe {
            match value {
                Value::Int(v) => gl.uniform_1_i32(location, *v),
                Value::Float(v) => gl.uniform_1_f32(location, *v),
                Value::IVec2(v) => gl.uniform_2_i32_slice(location, v),
                Value::Vec2(v) => gl.uniform_2_f32_slice(location, v),
                Value::Vec3(v) => gl.uniform_3_f32_slice(location, v),
                Value::Vec4(v) => gl.uniform_4_f32_slice(location, v),
                Value::Mat4(v) => gl.uniform_matrix_4_f32_slice(location, false, v),
                Value::Floats(v) => gl.uniform_1_f32_slice(location, v),
                Value::Ints(v) => gl.uniform_1_i32_slice(location, v),
                Value::Vec3s(v) => gl.uniform_3_f32_slice(location, v),
                Value::Vec4s(v) => gl.uniform_4_f32_slice(location, v),
            }
        }
    }

    /// Use this program and set `uniforms` on it.
    pub(crate) fn use_with(&mut self, gl: &glow::Context, uniforms: &[(&'static str, Value)]) {
        // SAFETY: this program is linked on the caller's context.
        unsafe { gl.use_program(Some(self.id)) };
        for (name, value) in uniforms {
            self.set(gl, name, value);
        }
    }

    /// Delete the program.
    pub(crate) fn delete(self, gl: &glow::Context) {
        // SAFETY: the program is this value's own.
        unsafe { gl.delete_program(self.id) };
    }
}

/// Compile both shaders and link them.
///
/// # Safety
///
/// Called on the caller's current context.
unsafe fn link(gl: &glow::Context, vertex: &str, fragment: &str) -> Result<glow::Program, String> {
    // SAFETY: objects created here are deleted or returned.
    unsafe {
        let program = gl.create_program()?;
        let mut shaders = Vec::new();
        for (kind, source) in [
            (glow::VERTEX_SHADER, vertex),
            (glow::FRAGMENT_SHADER, fragment),
        ] {
            let shader = gl.create_shader(kind)?;
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let log = gl.get_shader_info_log(shader);
                gl.delete_shader(shader);
                for s in shaders {
                    gl.delete_shader(s);
                }
                gl.delete_program(program);
                let which = if kind == glow::VERTEX_SHADER {
                    "vertex"
                } else {
                    "fragment"
                };
                return Err(format!("{which} shader: {log}"));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
        gl.link_program(program);
        let linked = gl.get_program_link_status(program);
        let log = gl.get_program_info_log(program);
        for s in shaders {
            gl.detach_shader(program, s);
            gl.delete_shader(s);
        }
        if linked {
            Ok(program)
        } else {
            gl.delete_program(program);
            Err(format!("link: {log}"))
        }
    }
}
