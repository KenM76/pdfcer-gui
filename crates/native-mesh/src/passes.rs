//! The passes around the main one: a depth map drawn first from another
//! viewpoint, and a two-pass effect over the finished frame.

use glow::HasContext as _;

use crate::buffers::{Buffers, ScreenTextures, complete, texture};
use crate::program::Program;
use crate::{Draw, Value};

/// The texture unit a depth map is bound to in the main pass.
const DEPTH_MAP_UNIT: u32 = 1;

/// A square depth texture and the framebuffer that writes it.
pub(crate) struct DepthTarget {
    pub(crate) size: i32,
    framebuffer: glow::Framebuffer,
    pub(crate) texture: glow::Texture,
}

impl DepthTarget {
    /// A `size`² 32-bit float depth texture.
    pub(crate) fn new(gl: &glow::Context, size: i32) -> Result<Self, String> {
        // SAFETY: objects created here are returned or deleted before an error.
        unsafe {
            let texture = texture(
                gl,
                glow::DEPTH_COMPONENT32F,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                [size, size],
            )?;
            let framebuffer = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            gl.draw_buffers(&[glow::NONE]);
            gl.read_buffer(glow::NONE);
            let target = Self {
                size,
                framebuffer,
                texture,
            };
            if let Err(e) = complete(gl, framebuffer, [size, size]) {
                target.delete(gl);
                return Err(e);
            }
            Ok(target)
        }
    }

    /// Clear to the far plane and draw the opaque `draws` with `program`,
    /// which must be in use with its uniforms set; the mesh's vertex array
    /// must be bound.
    pub(crate) fn draw(&self, gl: &glow::Context, draws: &[Draw]) {
        // SAFETY: the framebuffer is this target's own and complete.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.viewport(0, 0, self.size, self.size);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::BLEND);
            gl.disable(glow::CULL_FACE);
            gl.depth_mask(true);
            gl.clear_depth_f32(1.0);
            gl.clear(glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            for d in draws.iter().filter(|d| !d.translucent) {
                gl.draw_arrays(glow::TRIANGLES, d.first, d.count);
            }
        }
    }

    /// Bind the texture for the main pass and point `program`'s `sampler` at it.
    pub(crate) fn bind(&self, gl: &glow::Context, program: &mut Program, sampler: &'static str) {
        // SAFETY: the texture is this target's own.
        unsafe {
            gl.active_texture(glow::TEXTURE0 + DEPTH_MAP_UNIT);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.active_texture(glow::TEXTURE0);
        }
        program.set(gl, sampler, &Value::Int(DEPTH_MAP_UNIT as i32));
    }

    /// Delete both objects.
    pub(crate) fn delete(&self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.texture);
        }
    }
}

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
    /// second into `buffers.resolved`.
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
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(buffers.resolved));
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
