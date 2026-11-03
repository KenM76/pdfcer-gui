//! The two-pass effect over the finished faces.

use glow::HasContext as _;

use crate::Value;
use crate::buffers::{Buffers, ScreenTextures};
use crate::program::Program;

/// The two programs of a screen effect and the empty vertex array their
/// one triangle is drawn with.
pub(crate) struct ScreenPrograms {
    pub(crate) first: Program,
    pub(crate) second: Program,
    vao: glow::VertexArray,
}

impl ScreenPrograms {
    /// Link `vertex` with each fragment shader.
    pub(crate) fn new(
        gl: &glow::Context,
        vertex: &str,
        [first, second]: [&str; 2],
    ) -> Result<Self, String> {
        let first = Program::new(gl, vertex, first)?;
        let second = match Program::new(gl, vertex, second) {
            Ok(p) => p,
            Err(e) => {
                first.delete(gl);
                return Err(e);
            }
        };
        // SAFETY: on the caller's current context; kept in the returned value.
        let vao = match unsafe { gl.create_vertex_array() } {
            Ok(v) => v,
            Err(e) => {
                first.delete(gl);
                second.delete(gl);
                return Err(e);
            }
        };
        Ok(Self { first, second, vao })
    }

    /// Run the effect over the frame in `buffers`: copy its colour and depth
    /// into `textures`, draw the first pass into `textures.output`, and the
    /// second over `buffers.multisampled`'s colour, keeping its depth.
    pub(crate) fn run(
        &mut self,
        gl: &glow::Context,
        effect: &crate::ScreenEffect<'_>,
        buffers: &Buffers,
        textures: &ScreenTextures,
    ) {
        textures.take_frame(gl, buffers);
        let [w, h] = textures.size;
        // SAFETY: the framebuffers, textures and vertex array are the
        // mesh's own; units 0–2 are rebound to none before returning.
        unsafe {
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::BLEND);
            gl.depth_mask(false);
            gl.viewport(0, 0, w, h);
            gl.bind_vertex_array(Some(self.vao));
            for (unit, t) in [textures.depth, textures.colour, textures.output]
                .into_iter()
                .enumerate()
            {
                gl.active_texture(glow::TEXTURE0 + unit as u32);
                gl.bind_texture(glow::TEXTURE_2D, Some(t));
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(textures.effect));
            self.first.use_with(gl, effect.uniforms);
            self.first.set(gl, effect.depth, &Value::Int(0));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(buffers.multisampled));
            self.second.use_with(gl, effect.uniforms);
            self.second.set(gl, effect.depth, &Value::Int(0));
            self.second.set(gl, effect.colour, &Value::Int(1));
            self.second.set(gl, effect.first_output, &Value::Int(2));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            for unit in (0..3).rev() {
                gl.active_texture(glow::TEXTURE0 + unit);
                gl.bind_texture(glow::TEXTURE_2D, None);
            }
        }
    }

    /// Delete every object.
    pub(crate) fn delete(self, gl: &glow::Context) {
        // SAFETY: the vertex array is this value's own.
        unsafe { gl.delete_vertex_array(self.vao) };
        self.first.delete(gl);
        self.second.delete(gl);
    }
}
