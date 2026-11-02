//! The offscreen buffers a frame is drawn into, the textures a depth map or
//! a screen effect reads, and the copy onto the caller's framebuffer.

use glow::HasContext as _;

use crate::Target;

/// The multisampled buffer the draws go into and the plain one it resolves
/// into, which is read back and copied out.
pub(crate) struct Buffers {
    pub(crate) size: [i32; 2],
    pub(crate) multisampled: glow::Framebuffer,
    ms_colour: glow::Renderbuffer,
    ms_depth: glow::Renderbuffer,
    pub(crate) resolved: glow::Framebuffer,
    colour: glow::Renderbuffer,
}

/// The depth format of the multisampled buffer, and so of the depth texture
/// a screen effect reads: a depth blit needs the two to match.
const DEPTH_FORMAT: u32 = glow::DEPTH_COMPONENT24;

impl Buffers {
    /// Buffers `size` pixels with `samples` samples a pixel.
    pub(crate) fn new(gl: &glow::Context, size: [i32; 2], samples: i32) -> Result<Self, String> {
        let [w, h] = size;
        // SAFETY: objects created here are returned or deleted before an error.
        unsafe {
            let buffers = Self {
                size,
                ms_colour: gl.create_renderbuffer()?,
                ms_depth: gl.create_renderbuffer()?,
                colour: gl.create_renderbuffer()?,
                multisampled: gl.create_framebuffer()?,
                resolved: gl.create_framebuffer()?,
            };
            for (rb, samples, format) in [
                (buffers.ms_colour, samples, glow::RGBA8),
                (buffers.ms_depth, samples, DEPTH_FORMAT),
                (buffers.colour, 0, glow::RGBA8),
            ] {
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rb));
                gl.renderbuffer_storage_multisample(glow::RENDERBUFFER, samples, format, w, h);
            }
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            let attach = [
                (
                    buffers.multisampled,
                    glow::COLOR_ATTACHMENT0,
                    buffers.ms_colour,
                ),
                (
                    buffers.multisampled,
                    glow::DEPTH_ATTACHMENT,
                    buffers.ms_depth,
                ),
                (buffers.resolved, glow::COLOR_ATTACHMENT0, buffers.colour),
            ];
            for (fb, point, rb) in attach {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, point, glow::RENDERBUFFER, Some(rb));
            }
            for fb in [buffers.multisampled, buffers.resolved] {
                if let Err(e) = complete(gl, fb, size) {
                    buffers.delete(gl);
                    return Err(e);
                }
            }
            Ok(buffers)
        }
    }

    /// Delete every object.
    pub(crate) fn delete(&self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.multisampled);
            gl.delete_framebuffer(self.resolved);
            gl.delete_renderbuffer(self.ms_colour);
            gl.delete_renderbuffer(self.ms_depth);
            gl.delete_renderbuffer(self.colour);
        }
    }

    /// Average the multisampled colour into `resolved`.
    pub(crate) fn resolve(&self, gl: &glow::Context) {
        blit(
            gl,
            self.multisampled,
            self.resolved,
            self.size,
            glow::COLOR_BUFFER_BIT,
        );
    }
}

/// The textures a screen effect reads and writes, viewport-sized: the
/// frame's colour and depth copied out of the multisampled buffer, and the
/// first pass's output.
pub(crate) struct ScreenTextures {
    pub(crate) size: [i32; 2],
    /// Holds `colour` and `depth`.
    pub(crate) frame: glow::Framebuffer,
    pub(crate) colour: glow::Texture,
    pub(crate) depth: glow::Texture,
    /// Holds `output`.
    pub(crate) effect: glow::Framebuffer,
    pub(crate) output: glow::Texture,
}

impl ScreenTextures {
    /// Textures `size` pixels.
    pub(crate) fn new(gl: &glow::Context, size: [i32; 2]) -> Result<Self, String> {
        let [w, h] = size;
        // SAFETY: objects created here are returned or deleted before an error.
        unsafe {
            let textures = Self {
                size,
                frame: gl.create_framebuffer()?,
                colour: texture(gl, glow::RGBA8, glow::RGBA, glow::UNSIGNED_BYTE, [w, h])?,
                depth: texture(
                    gl,
                    DEPTH_FORMAT,
                    glow::DEPTH_COMPONENT,
                    glow::UNSIGNED_INT,
                    [w, h],
                )?,
                effect: gl.create_framebuffer()?,
                output: texture(gl, glow::RGBA8, glow::RGBA, glow::UNSIGNED_BYTE, [w, h])?,
            };
            let attach = [
                (textures.frame, glow::COLOR_ATTACHMENT0, textures.colour),
                (textures.frame, glow::DEPTH_ATTACHMENT, textures.depth),
                (textures.effect, glow::COLOR_ATTACHMENT0, textures.output),
            ];
            for (fb, point, t) in attach {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                gl.framebuffer_texture_2d(glow::FRAMEBUFFER, point, glow::TEXTURE_2D, Some(t), 0);
            }
            for fb in [textures.frame, textures.effect] {
                if let Err(e) = complete(gl, fb, size) {
                    textures.delete(gl);
                    return Err(e);
                }
            }
            Ok(textures)
        }
    }

    /// Copy the multisampled colour and depth of `buffers` into `frame`.
    pub(crate) fn take_frame(&self, gl: &glow::Context, buffers: &Buffers) {
        let size = self.size;
        blit(
            gl,
            buffers.multisampled,
            self.frame,
            size,
            glow::COLOR_BUFFER_BIT,
        );
        blit(
            gl,
            buffers.multisampled,
            self.frame,
            size,
            glow::DEPTH_BUFFER_BIT,
        );
    }

    /// Delete every object.
    pub(crate) fn delete(&self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.frame);
            gl.delete_framebuffer(self.effect);
            gl.delete_texture(self.colour);
            gl.delete_texture(self.depth);
            gl.delete_texture(self.output);
        }
    }
}

/// A `size` texture of `internal` format, sampled `NEAREST`, clamped, with
/// no compare mode.
///
/// # Safety
///
/// Called on the caller's current context; the texture is the caller's.
pub(crate) unsafe fn texture(
    gl: &glow::Context,
    internal: u32,
    format: u32,
    kind: u32,
    [w, h]: [i32; 2],
) -> Result<glow::Texture, String> {
    // SAFETY: the texture is created here and returned.
    unsafe {
        let t = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(t));
        for (name, value) in [
            (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
            (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
            (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
            (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
            (glow::TEXTURE_COMPARE_MODE, glow::NONE),
        ] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, name, value as i32);
        }
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            internal as i32,
            w,
            h,
            0,
            format,
            kind,
            glow::PixelUnpackData::Slice(None),
        );
        gl.bind_texture(glow::TEXTURE_2D, None);
        Ok(t)
    }
}

/// Fail unless `fb` is complete; leaves no framebuffer bound.
pub(crate) fn complete(
    gl: &glow::Context,
    fb: glow::Framebuffer,
    [w, h]: [i32; 2],
) -> Result<(), String> {
    // SAFETY: a status query on the caller's framebuffer.
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
        let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        if status == glow::FRAMEBUFFER_COMPLETE {
            Ok(())
        } else {
            Err(format!("offscreen buffer {w}x{h} incomplete: {status:#x}"))
        }
    }
}

/// Copy `mask` of `size` pixels from `from` to `to`, unscaled.
fn blit(
    gl: &glow::Context,
    from: glow::Framebuffer,
    to: glow::Framebuffer,
    [w, h]: [i32; 2],
    mask: u32,
) {
    // SAFETY: both framebuffers are the caller's own and complete.
    unsafe {
        gl.disable(glow::SCISSOR_TEST);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(from));
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(to));
        gl.blit_framebuffer(0, 0, w, h, 0, 0, w, h, mask, glow::NEAREST);
    }
}

/// Read `resolved` back when asked, and copy it to the target's viewport,
/// clipped; leaves the destination bound.
pub(crate) fn present(gl: &glow::Context, b: &Buffers, target: &Target) -> Option<Vec<u8>> {
    let [w, h] = b.size;
    // SAFETY: the resolved framebuffer is the mesh's own and complete, and
    // bound for reading when `read` runs; the destination is the caller's.
    unsafe {
        gl.depth_mask(true);
        gl.disable(glow::DEPTH_TEST);
        gl.disable(glow::BLEND);
        gl.bind_vertex_array(None);
        gl.use_program(None);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(b.resolved));
        let pixels = target.read_back.then(|| read(gl, w, h));
        let [x, y, _, _] = target.viewport;
        let [cx, cy, cw, ch] = target.clip;
        gl.enable(glow::SCISSOR_TEST);
        gl.scissor(cx, cy, cw, ch);
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, target.destination);
        gl.blit_framebuffer(
            0,
            0,
            w,
            h,
            x,
            y,
            x + w,
            y + h,
            glow::COLOR_BUFFER_BIT,
            glow::NEAREST,
        );
        gl.bind_framebuffer(glow::FRAMEBUFFER, target.destination);
        pixels
    }
}

/// The bound read framebuffer's `w × h` pixels, top row first.
///
/// # Safety
///
/// A complete framebuffer at least `w × h` is bound for reading.
unsafe fn read(gl: &glow::Context, w: i32, h: i32) -> Vec<u8> {
    let row = w as usize * 4;
    let mut pixels = vec![0_u8; row * h as usize];
    // SAFETY: the slice is exactly `w × h × 4` bytes at pack alignment 4.
    unsafe {
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
        gl.read_pixels(
            0,
            0,
            w,
            h,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut pixels)),
        );
    }
    let mut flipped = Vec::with_capacity(pixels.len());
    for r in pixels.chunks_exact(row).rev() {
        flipped.extend_from_slice(r);
    }
    flipped
}
