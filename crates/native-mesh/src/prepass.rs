//! The depth prepass: the opaque draws' nearest depth, drawn into a
//! single-sample texture before the main pass, which samples it to skip
//! work on fragments something opaque hides.
//!
//! The texture is the main buffer's size in pixels with the main pass's
//! viewport at the origin, so a main-pass `gl_FragCoord` indexes it
//! directly. It is 24-bit (16-bit rounding would hide visible fragments
//! from a shader that compares within a fine tolerance), `NEAREST`, with no
//! compare mode, and depth-only: colour is never written.

use glow::HasContext as _;

use crate::buffers::{complete, texture};
use crate::program::Program;
use crate::{Draw, Prepass};

/// The prepass's framebuffer and the depth texture it writes.
pub(crate) struct PrepassTarget {
    pub(crate) size: [i32; 2],
    framebuffer: glow::Framebuffer,
    pub(crate) depth: glow::Texture,
}

impl PrepassTarget {
    /// A target `size` pixels.
    pub(crate) fn new(gl: &glow::Context, size: [i32; 2]) -> Result<Self, String> {
        // SAFETY: objects created here are returned or deleted before an error.
        unsafe {
            let target = Self {
                size,
                framebuffer: gl.create_framebuffer()?,
                depth: texture(
                    gl,
                    glow::DEPTH_COMPONENT24,
                    glow::DEPTH_COMPONENT,
                    glow::UNSIGNED_INT,
                    size,
                )?,
            };
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(target.depth),
                0,
            );
            gl.draw_buffers(&[glow::NONE]);
            gl.read_buffer(glow::NONE);
            if let Err(e) = complete(gl, target.framebuffer, size) {
                target.delete(gl);
                return Err(e);
            }
            Ok(target)
        }
    }

    /// Clear to 1.0 and draw every opaque draw with `program` and the
    /// prepass's uniforms, `LESS`, colour masked, no culling. The caller's
    /// vertex array must be bound. Leaves the colour mask off.
    pub(crate) fn draw(
        &self,
        gl: &glow::Context,
        program: &mut Program,
        prepass: &Prepass<'_>,
        draws: &[Draw],
    ) {
        let [w, h] = self.size;
        // SAFETY: the framebuffer is this target's own and complete.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.viewport(0, 0, w, h);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::BLEND);
            gl.disable(glow::CULL_FACE);
            gl.color_mask(false, false, false, false);
            gl.depth_mask(true);
            gl.clear_depth_f32(1.0);
            gl.clear(glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
        }
        program.use_with(gl, prepass.uniforms);
        for d in draws.iter().filter(|d| !d.translucent) {
            // SAFETY: the range is the caller's, as for the main pass.
            unsafe { gl.draw_arrays(glow::TRIANGLES, d.first, d.count) };
        }
    }

    /// Delete every object.
    pub(crate) fn delete(&self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.depth);
        }
    }
}
