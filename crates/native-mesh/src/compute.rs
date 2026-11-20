//! Fragment passes that compute into float targets and are read back: the
//! caller's programs drawn as one triangle over a whole `RGBA32F` target,
//! reading `RGBA32F` tables, `RGBA8UI` layered textures and earlier targets
//! with `texelFetch`.
//!
//! Contract: a [`Compute`] is made and used on the context the caller paints
//! with, and [`Compute::destroy`]ed on it. [`Compute::draw`] limits a pass to
//! a band of rows with the scissor test, so a caller bounded by a GPU
//! watchdog spreads a long pass over several calls and frames. Needs
//! `EXT_color_buffer_float` ([`Compute::supported`]).
//!
//! State left behind: no framebuffer, program, vertex array or texture
//! bound on the units used, scissor and blending off. `egui_glow`
//! re-prepares its own state after a callback, and [`crate::Mesh::draw`]
//! sets every state it reads.

use glow::HasContext as _;

use crate::Value;
use crate::program::Program;

/// An input a pass reads.
#[derive(Debug, Clone, Copy)]
pub enum Input<'a> {
    /// An `RGBA32F` 2D texture `width` texels across, four floats a texel,
    /// row 0 first, a whole number of rows.
    Floats {
        /// Texels a row.
        width: u32,
        /// The floats.
        data: &'a [f32],
    },
    /// An `RGBA8UI` `TEXTURE_2D_ARRAY`, four bytes a texel, layer 0 row 0
    /// first.
    Bytes {
        /// Texels a row.
        width: u32,
        /// Rows a layer.
        height: u32,
        /// Layers, at least 1.
        layers: u32,
        /// The bytes.
        data: &'a [u8],
    },
}

/// What a sampler of a pass is bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bind {
    /// The input at this index of [`Compute::new`]'s `inputs`.
    Input(usize),
    /// The target at this index of [`Compute::new`]'s `targets`.
    Target(usize),
}

/// The programs, inputs and targets of one computation.
pub struct Compute {
    programs: Vec<Program>,
    inputs: Vec<(glow::Texture, u32)>,
    targets: Vec<FloatTarget>,
    vao: glow::VertexArray,
}

/// An `RGBA32F` texture and the framebuffer drawing into it.
struct FloatTarget {
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
    size: [i32; 2],
}

impl Compute {
    /// Whether the context can render into `RGBA32F` targets.
    #[must_use]
    pub fn supported(gl: &glow::Context) -> bool {
        gl.supported_extensions().contains("EXT_color_buffer_float")
            || gl
                .supported_extensions()
                .contains("GL_EXT_color_buffer_float")
            || gl.version().major >= 3 && !gl.version().is_embedded
    }

    /// Link one program per fragment shader over `vertex`, upload `inputs`
    /// and allocate `targets` (`[width, height]` each).
    ///
    /// # Errors
    ///
    /// A shader that does not compile or link, a target the driver will not
    /// complete, or an input whose length is not a whole number of rows.
    pub fn new(
        gl: &glow::Context,
        vertex: &str,
        fragments: &[&str],
        inputs: &[Input<'_>],
        targets: &[[u32; 2]],
    ) -> Result<Self, String> {
        // SAFETY: on the caller's context; kept in the result or deleted.
        let vao = unsafe { gl.create_vertex_array()? };
        let mut made = Self {
            programs: Vec::new(),
            inputs: Vec::new(),
            targets: Vec::new(),
            vao,
        };
        if let Err(error) = made.fill(gl, vertex, fragments, inputs, targets) {
            made.destroy(gl);
            return Err(error);
        }
        Ok(made)
    }

    fn fill(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragments: &[&str],
        inputs: &[Input<'_>],
        targets: &[[u32; 2]],
    ) -> Result<(), String> {
        for fragment in fragments {
            self.programs.push(Program::new(gl, vertex, fragment)?);
        }
        for input in inputs {
            self.inputs.push(upload(gl, input)?);
        }
        for size in targets {
            self.targets.push(FloatTarget::new(gl, *size)?);
        }
        Ok(())
    }

    /// Draw `program` over rows `rows` of `target`, each sampler in
    /// `samplers` bound to its source on its own unit (from 0, in order).
    /// A target may not be both drawn and sampled. Rows past the target are
    /// clipped; nothing is drawn for an empty range.
    pub fn draw(
        &mut self,
        gl: &glow::Context,
        program: usize,
        target: usize,
        rows: std::ops::Range<u32>,
        samplers: &[(&'static str, Bind)],
        uniforms: &[(&'static str, Value)],
    ) {
        let (Some(t), Some(p)) = (self.targets.get(target), self.programs.get_mut(program)) else {
            return;
        };
        let [width, height] = t.size;
        let low = (rows.start as i32).min(height);
        let high = (rows.end as i32).min(height);
        if high <= low {
            return;
        }
        let mut bound = Vec::with_capacity(samplers.len());
        p.use_with(gl, uniforms);
        // SAFETY: every object named is this value's own, on the caller's
        // context; the state set here is reset below.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(t.framebuffer));
            gl.viewport(0, 0, width, height);
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(0, low, width, high - low);
            gl.disable(glow::BLEND);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            for (unit, (name, source)) in samplers.iter().enumerate() {
                let (texture, kind) = match source {
                    Bind::Input(i) => match self.inputs.get(*i) {
                        Some(t) => *t,
                        None => continue,
                    },
                    Bind::Target(i) if *i == target => continue,
                    Bind::Target(i) => match self.targets.get(*i) {
                        Some(t) => (t.texture, glow::TEXTURE_2D),
                        None => continue,
                    },
                };
                gl.active_texture(glow::TEXTURE0 + unit as u32);
                gl.bind_texture(kind, Some(texture));
                bound.push((unit as u32, kind));
                p.set(gl, name, &Value::Int(unit as i32));
            }
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.flush();
            gl.bind_vertex_array(None);
            for (unit, kind) in bound {
                gl.active_texture(glow::TEXTURE0 + unit);
                gl.bind_texture(kind, None);
            }
            gl.active_texture(glow::TEXTURE0);
            gl.disable(glow::SCISSOR_TEST);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.use_program(None);
        }
    }

    /// Wait for every pass drawn so far to finish.
    pub fn finish(&self, gl: &glow::Context) {
        // SAFETY: a plain synchronisation call on the caller's context.
        unsafe { gl.finish() };
    }

    /// `target`'s floats, four a texel, row 0 first; empty for no target.
    #[must_use]
    pub fn read(&self, gl: &glow::Context, target: usize) -> Vec<f32> {
        let Some(t) = self.targets.get(target) else {
            return Vec::new();
        };
        let [width, height] = t.size;
        let mut bytes = vec![0_u8; width as usize * height as usize * 16];
        // SAFETY: the framebuffer is this value's own and `bytes` holds
        // exactly `width × height` RGBA float texels.
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(t.framebuffer));
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
            gl.read_pixels(
                0,
                0,
                width,
                height,
                glow::RGBA,
                glow::FLOAT,
                glow::PixelPackData::Slice(Some(&mut bytes)),
            );
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
        bytes
            .chunks_exact(4)
            .map(|b| f32::from_ne_bytes([b[0], b[1], b[2], b[3]]))
            .collect()
    }

    /// Delete every GL object.
    pub fn destroy(self, gl: &glow::Context) {
        for p in self.programs {
            p.delete(gl);
        }
        // SAFETY: the objects are this value's own.
        unsafe {
            for (texture, _) in self.inputs {
                gl.delete_texture(texture);
            }
            for t in self.targets {
                gl.delete_framebuffer(t.framebuffer);
                gl.delete_texture(t.texture);
            }
            gl.delete_vertex_array(self.vao);
        }
    }
}

impl FloatTarget {
    fn new(gl: &glow::Context, [width, height]: [u32; 2]) -> Result<Self, String> {
        let size = [width.max(1) as i32, height.max(1) as i32];
        // SAFETY: on the caller's context; both objects are returned or
        // deleted.
        unsafe {
            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            nearest(gl, glow::TEXTURE_2D);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA32F as i32,
                size[0],
                size[1],
                0,
                glow::RGBA,
                glow::FLOAT,
                glow::PixelUnpackData::Slice(None),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            let framebuffer = match gl.create_framebuffer() {
                Ok(f) => f,
                Err(e) => {
                    gl.delete_texture(texture);
                    return Err(e);
                }
            };
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.delete_framebuffer(framebuffer);
                gl.delete_texture(texture);
                return Err(format!("float target incomplete: 0x{status:x}"));
            }
            Ok(Self {
                texture,
                framebuffer,
                size,
            })
        }
    }
}

/// Upload one input; its texture and binding target.
fn upload(gl: &glow::Context, input: &Input<'_>) -> Result<(glow::Texture, u32), String> {
    match *input {
        Input::Floats { width, data } => {
            let row = width as usize * 4;
            if width == 0 || data.is_empty() || data.len() % row != 0 {
                return Err(format!(
                    "a float input of {} is not rows of {width}",
                    data.len()
                ));
            }
            let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_ne_bytes()).collect();
            // SAFETY: on the caller's context; `bytes` is `width × rows × 16`.
            unsafe {
                let texture = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                nearest(gl, glow::TEXTURE_2D);
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA32F as i32,
                    width as i32,
                    (data.len() / row) as i32,
                    0,
                    glow::RGBA,
                    glow::FLOAT,
                    glow::PixelUnpackData::Slice(Some(&bytes)),
                );
                gl.bind_texture(glow::TEXTURE_2D, None);
                Ok((texture, glow::TEXTURE_2D))
            }
        }
        Input::Bytes {
            width,
            height,
            layers,
            data,
        } => {
            let want = width as usize * height as usize * layers as usize * 4;
            if want == 0 || data.len() != want {
                return Err(format!("a byte input of {} is not {want}", data.len()));
            }
            // SAFETY: on the caller's context; `data` is exactly the array.
            unsafe {
                let texture = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
                nearest(gl, glow::TEXTURE_2D_ARRAY);
                gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
                gl.tex_image_3d(
                    glow::TEXTURE_2D_ARRAY,
                    0,
                    glow::RGBA8UI as i32,
                    width as i32,
                    height as i32,
                    layers as i32,
                    0,
                    glow::RGBA_INTEGER,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
                Ok((texture, glow::TEXTURE_2D_ARRAY))
            }
        }
    }
}

/// Nearest filtering, clamped, no mipmaps, on the bound `target`.
///
/// # Safety
///
/// A texture is bound to `target` on the caller's context.
pub(crate) unsafe fn nearest(gl: &glow::Context, target: u32) {
    for (name, value) in [
        (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
        (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
        (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
        (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
    ] {
        // SAFETY: the caller's bound texture.
        unsafe { gl.tex_parameter_i32(target, name, value as i32) };
    }
}
