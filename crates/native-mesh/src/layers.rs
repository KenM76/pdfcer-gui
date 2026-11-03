//! The textures filled before the main pass ([`Layers`]) and the float
//! tables bound beside them ([`Table`]), each kept across frames on its own
//! texture unit.

use glow::HasContext as _;

use crate::buffers::complete;
use crate::program::Program;
use crate::{Draw, Fill, Format, Layers, Source, Table, Value};

/// A [`Layers`] texture, its framebuffer, and what it was filled from.
pub(crate) struct LayerTarget {
    shape: Shape,
    framebuffer: glow::Framebuffer,
    texture: glow::Texture,
    /// The mesh generation it was last filled at; `None` before the first.
    filled: Option<u64>,
}

/// What a texture must be recreated for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shape {
    array: bool,
    format: Format,
    size: i32,
    layers: i32,
}

impl Shape {
    fn of(spec: &Layers<'_>) -> Self {
        Self {
            array: spec.array,
            format: spec.format,
            size: spec.size.max(1),
            layers: if spec.array { spec.layers.max(1) } else { 1 },
        }
    }

    fn bind_point(self) -> u32 {
        if self.array {
            glow::TEXTURE_2D_ARRAY
        } else {
            glow::TEXTURE_2D
        }
    }

    fn attachment(self) -> u32 {
        match self.format {
            Format::Depth => glow::DEPTH_ATTACHMENT,
            Format::Coverage => glow::COLOR_ATTACHMENT0,
        }
    }
}

impl LayerTarget {
    /// The target `spec` asks for, keeping `old` when its shape matches.
    pub(crate) fn fit(
        gl: &glow::Context,
        old: Option<Self>,
        spec: &Layers<'_>,
    ) -> Result<Self, String> {
        let shape = Shape::of(spec);
        match old {
            Some(t) if t.shape == shape => Ok(t),
            other => {
                if let Some(t) = other {
                    t.delete(gl);
                }
                Self::new(gl, shape)
            }
        }
    }

    fn new(gl: &glow::Context, shape: Shape) -> Result<Self, String> {
        let point = shape.bind_point();
        let (internal, format, kind) = match shape.format {
            Format::Depth => (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT),
            Format::Coverage => (glow::R8, glow::RED, glow::UNSIGNED_BYTE),
        };
        let Shape { size, layers, .. } = shape;
        // SAFETY: objects created here are returned or deleted before an error.
        unsafe {
            let texture = gl.create_texture()?;
            gl.bind_texture(point, Some(texture));
            for (name, value) in [
                (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
                (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
                (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
                (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
                (glow::TEXTURE_COMPARE_MODE, glow::NONE),
            ] {
                gl.tex_parameter_i32(point, name, value as i32);
            }
            let empty = glow::PixelUnpackData::Slice(None);
            if shape.array {
                let i = internal as i32;
                gl.tex_image_3d(point, 0, i, size, size, layers, 0, format, kind, empty);
            } else {
                gl.tex_image_2d(
                    point,
                    0,
                    internal as i32,
                    size,
                    size,
                    0,
                    format,
                    kind,
                    empty,
                );
            }
            gl.bind_texture(point, None);
            let framebuffer = match gl.create_framebuffer() {
                Ok(f) => f,
                Err(e) => {
                    gl.delete_texture(texture);
                    return Err(e);
                }
            };
            let target = Self {
                shape,
                framebuffer,
                texture,
                filled: None,
            };
            target.attach(gl, 0);
            if let Err(e) = complete(gl, framebuffer, [size, size]) {
                target.delete(gl);
                return Err(e);
            }
            Ok(target)
        }
    }

    /// Bind the framebuffer with `layer` attached and the draw buffers set.
    fn attach(&self, gl: &glow::Context, layer: i32) {
        let point = self.shape.attachment();
        // SAFETY: the framebuffer and texture are this target's own.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            if self.shape.array {
                gl.framebuffer_texture_layer(
                    glow::FRAMEBUFFER,
                    point,
                    Some(self.texture),
                    0,
                    layer,
                );
            } else {
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    point,
                    glow::TEXTURE_2D,
                    Some(self.texture),
                    0,
                );
            }
            match self.shape.format {
                Format::Depth => gl.draw_buffers(&[glow::NONE]),
                Format::Coverage => gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]),
            }
            gl.read_buffer(glow::NONE);
        }
    }

    /// Whether `spec` asks for a fill at mesh `generation`.
    pub(crate) fn wants_fill(&self, spec: &Layers<'_>, generation: u64) -> bool {
        spec.redraw || self.filled != Some(generation)
    }

    /// Clear every layer, then run `spec`'s fills with `programs`; the
    /// mesh's vertex array must be bound and `pictures` are its pictures.
    pub(crate) fn fill(
        &mut self,
        gl: &glow::Context,
        spec: &Layers<'_>,
        programs: &mut [Program],
        draws: &[Draw],
        pictures: &[glow::Texture],
        generation: u64,
    ) -> Result<(), String> {
        let size = self.shape.size;
        // SAFETY: state changes on the caller's current context.
        unsafe {
            gl.viewport(0, 0, size, size);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::CULL_FACE);
        }
        for layer in 0..self.shape.layers {
            self.attach(gl, layer);
            self.clear(gl);
        }
        for fill in spec.fills {
            let Some(program) = programs.get_mut(fill.program) else {
                return Err(format!(
                    "fill names program {} of {}",
                    fill.program,
                    programs.len()
                ));
            };
            self.attach(gl, fill.layer.clamp(0, self.shape.layers - 1));
            self.state(gl);
            program.use_with(gl, fill.uniforms);
            run(gl, program, fill, draws, pictures);
        }
        // SAFETY: restores the default blend and colour mask.
        unsafe {
            gl.disable(glow::BLEND);
            gl.color_mask(true, true, true, true);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
        self.filled = Some(generation);
        Ok(())
    }

    /// Clear the attached layer to its empty value.
    fn clear(&self, gl: &glow::Context) {
        // SAFETY: the bound framebuffer is this target's own.
        unsafe {
            match self.shape.format {
                Format::Depth => {
                    gl.depth_mask(true);
                    gl.clear_depth_f32(1.0);
                    gl.clear(glow::DEPTH_BUFFER_BIT);
                }
                Format::Coverage => {
                    gl.color_mask(true, true, true, true);
                    gl.clear_color(1.0, 1.0, 1.0, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }
            }
        }
    }

    /// The fill state for this target's format.
    fn state(&self, gl: &glow::Context) {
        // SAFETY: state changes on the caller's current context.
        unsafe {
            match self.shape.format {
                Format::Depth => {
                    gl.disable(glow::BLEND);
                    gl.color_mask(false, false, false, false);
                    gl.depth_mask(true);
                    gl.enable(glow::DEPTH_TEST);
                    gl.depth_func(glow::LESS);
                }
                Format::Coverage => {
                    gl.disable(glow::DEPTH_TEST);
                    gl.depth_mask(false);
                    gl.color_mask(true, true, true, true);
                    gl.enable(glow::BLEND);
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::ZERO, glow::SRC_COLOR);
                }
            }
        }
    }

    /// Bind the texture on `spec.unit` and point the main program at it.
    pub(crate) fn bind(&self, gl: &glow::Context, main: &mut Program, spec: &Layers<'_>) {
        // SAFETY: the texture is this target's own.
        unsafe {
            gl.active_texture(glow::TEXTURE0 + spec.unit);
            gl.bind_texture(self.shape.bind_point(), Some(self.texture));
            gl.active_texture(glow::TEXTURE0);
        }
        main.set(gl, spec.sampler, &Value::Int(spec.unit as i32));
    }

    /// Unbind the texture from `unit`.
    pub(crate) fn unbind(&self, gl: &glow::Context, unit: u32) {
        // SAFETY: a binding change on the caller's current context.
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(self.shape.bind_point(), None);
            gl.active_texture(glow::TEXTURE0);
        }
    }

    /// Delete both objects.
    pub(crate) fn delete(self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.texture);
        }
    }
}

/// Draw `fill`'s draws with `program`, in use.
fn run(
    gl: &glow::Context,
    program: &mut Program,
    fill: &Fill<'_>,
    draws: &[Draw],
    pictures: &[glow::Texture],
) {
    let translucent = fill.source == Source::Translucent;
    for d in draws.iter().filter(|d| d.translucent == translucent) {
        if translucent {
            for (name, value) in &d.uniforms {
                program.set(gl, name, value);
            }
        }
        let texture = d.picture.and_then(|i| pictures.get(i).copied());
        // SAFETY: the texture is the mesh's own or none; the range is the
        // caller's, and GL raises an error rather than read past the buffer.
        unsafe {
            if translucent {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, texture);
            }
            gl.draw_arrays(glow::TRIANGLES, d.first, d.count);
        }
    }
}

/// A [`Table`]'s texture and the floats it holds.
pub(crate) struct TableTexture {
    texture: glow::Texture,
    data: Vec<f32>,
}

impl TableTexture {
    /// The texture `spec` asks for, re-uploading `old` only when the floats
    /// differ.
    pub(crate) fn fit(
        gl: &glow::Context,
        old: Option<Self>,
        spec: &Table<'_>,
    ) -> Result<Self, String> {
        let width = spec.width.max(1);
        let row = width as usize * 4;
        if spec.data.is_empty() || !spec.data.len().is_multiple_of(row) {
            if let Some(t) = old {
                t.delete(gl);
            }
            return Err(format!(
                "table of {} floats is not rows of {width} texels",
                spec.data.len()
            ));
        }
        let texture = match old {
            Some(t) if t.data == spec.data => return Ok(t),
            Some(t) => t.texture,
            // SAFETY: on the caller's current context; kept in the result.
            None => unsafe { gl.create_texture()? },
        };
        let rows = (spec.data.len() / row) as i32;
        let bytes: Vec<u8> = spec.data.iter().flat_map(|f| f.to_ne_bytes()).collect();
        // SAFETY: the texture is this value's own; the slice is exactly
        // `width × rows × 16` bytes.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            for (name, value) in [
                (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
                (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
                (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
                (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
            ] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, name, value as i32);
            }
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA32F as i32,
                width,
                rows,
                0,
                glow::RGBA,
                glow::FLOAT,
                glow::PixelUnpackData::Slice(Some(&bytes)),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
        Ok(Self {
            texture,
            data: spec.data.to_vec(),
        })
    }

    /// Bind the texture on `spec.unit` and point the main program at it.
    pub(crate) fn bind(&self, gl: &glow::Context, main: &mut Program, spec: &Table<'_>) {
        // SAFETY: the texture is this value's own.
        unsafe {
            gl.active_texture(glow::TEXTURE0 + spec.unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.active_texture(glow::TEXTURE0);
        }
        main.set(gl, spec.sampler, &Value::Int(spec.unit as i32));
    }

    /// Delete the texture.
    pub(crate) fn delete(self, gl: &glow::Context) {
        // SAFETY: the texture is this value's own.
        unsafe { gl.delete_texture(self.texture) };
    }
}
