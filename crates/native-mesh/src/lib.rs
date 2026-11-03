//! Draw a depth-tested triangle mesh with a caller's shaders into an
//! offscreen multisampled buffer, then copy it onto the framebuffer an
//! `egui_glow` paint callback runs in.
//!
//! Contract: every call takes the `glow::Context` the caller paints with, on
//! the thread that owns it. Vertices are interleaved `f32`s; attribute `i`
//! sits at shader location `i` with `attributes[i]` components. Uniforms are
//! named, so this crate knows no shader. A [`Mesh`] owns GL objects and must
//! be [`Mesh::destroy`]ed with the same context.
//!
//! Optional passes surround the main one ([`Passes`]), in this order: float
//! tables and layered textures filled from other viewpoints, bound to the
//! main program as samplers; the main draws; a two-pass effect over the
//! faces' colour and depth; the opaque draws repeated and added in; and
//! overlays with their own vertices, depth-tested against the faces. The
//! caller assigns every sampler its texture unit, picture on unit 0.
//!
//! State left behind: the destination framebuffer bound, no program, no
//! vertex array, depth test, blending and culling off, scissor on with the
//! clip box. `egui_glow` re-prepares its own state after a callback.

mod buffers;
mod layers;
mod overlay;
mod passes;
mod program;
mod spec;

use glow::HasContext as _;

use buffers::{Buffers, ScreenTextures};
use layers::{LayerTarget, TableTexture};
use overlay::OverlayProgram;
use passes::ScreenPrograms;
use program::Program;
pub use spec::{
    Attribute, Draw, Fill, Format, Layers, Layout, Overlay, Passes, Picture, Primitive, Repeat,
    ScreenEffect, Source, Table, Target, Value, Wrap,
};

/// A compiled program, its geometry and pictures, the passes' programs and
/// textures, and its buffers.
pub struct Mesh {
    main: Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    pictures: Vec<glow::Texture>,
    buffers: Option<Buffers>,
    samples: i32,
    /// Programs a [`Fill`] names; they read the main attributes.
    programs: Vec<Program>,
    /// Parallel to the last [`Passes::layers`].
    layer_targets: Vec<LayerTarget>,
    /// Parallel to the last [`Passes::tables`].
    tables: Vec<TableTexture>,
    /// Bumped whenever the vertices or pictures change.
    generation: u64,
    screen_programs: Option<ScreenPrograms>,
    screen_textures: Option<ScreenTextures>,
    overlays: Vec<OverlayProgram>,
    widest_point: f32,
}

/// The most samples a pixel takes; the driver's limit if lower.
const SAMPLES: i32 = 4;

impl Mesh {
    /// Compile and link `vertex` and `fragment`, and lay out the vertex
    /// attributes `attributes` describes.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log, or the object it could not create.
    pub fn new(
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
        attributes: &[i32],
    ) -> Result<Self, String> {
        let main = Program::new(gl, vertex, fragment)?;
        // SAFETY: every call is on the caller's current context; objects
        // created here are owned by the returned `Mesh`.
        unsafe {
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let stride = attributes.iter().sum::<i32>() * 4;
            let mut offset = 0;
            for (location, &size) in (0_u32..).zip(attributes) {
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_pointer_f32(location, size, glow::FLOAT, false, stride, offset);
                offset += size * 4;
            }
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            let samples = gl.get_parameter_i32(glow::MAX_SAMPLES).clamp(0, SAMPLES);
            Ok(Self {
                main,
                vao,
                vbo,
                pictures: Vec::new(),
                buffers: None,
                samples,
                programs: Vec::new(),
                layer_targets: Vec::new(),
                tables: Vec::new(),
                generation: 0,
                screen_programs: None,
                screen_textures: None,
                overlays: Vec::new(),
                widest_point: overlay::widest_point(gl),
            })
        }
    }

    /// Link a program a [`Fill`] draws with; it reads the same vertex
    /// attributes as the main program. Returns its index.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log.
    pub fn add_program(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
    ) -> Result<usize, String> {
        self.programs.push(Program::new(gl, vertex, fragment)?);
        Ok(self.programs.len() - 1)
    }

    /// Link an [`Overlay`]'s program over its own vertices, laid out as
    /// `layout`. Returns its index.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log, or the object it could not create.
    pub fn add_overlay(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
        layout: &Layout<'_>,
    ) -> Result<usize, String> {
        self.overlays
            .push(OverlayProgram::new(gl, vertex, fragment, layout)?);
        Ok(self.overlays.len() - 1)
    }

    /// Replace overlay `index`'s vertices; an unknown index is ignored.
    pub fn set_overlay_vertices(&mut self, gl: &glow::Context, index: usize, vertices: &[f32]) {
        if let Some(o) = self.overlays.get(index) {
            o.set_vertices(gl, vertices);
        }
    }

    /// The widest point, in pixels, the context draws.
    #[must_use]
    pub fn widest_point(&self) -> f32 {
        self.widest_point
    }

    /// Link a [`ScreenEffect`]'s two programs, both with `vertex`, which
    /// reads no attributes and places three vertices.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log, or the object it could not create.
    pub fn set_screen_programs(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragments: [&str; 2],
    ) -> Result<(), String> {
        let programs = ScreenPrograms::new(gl, vertex, fragments)?;
        if let Some(old) = self.screen_programs.replace(programs) {
            old.delete(gl);
        }
        Ok(())
    }

    /// Replace the vertices every [`Draw`] indexes.
    pub fn set_vertices(&mut self, gl: &glow::Context, vertices: &[f32]) {
        let bytes: Vec<u8> = vertices.iter().flat_map(|f| f.to_ne_bytes()).collect();
        // SAFETY: the buffer is this mesh's own; the slice outlives the call.
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        self.generation += 1;
    }

    /// Replace the pictures [`Draw::picture`] indexes.
    ///
    /// # Errors
    ///
    /// A texture the driver could not create, or a picture whose bytes do
    /// not match its size.
    pub fn set_pictures(&mut self, gl: &glow::Context, pictures: &[Picture]) -> Result<(), String> {
        self.drop_pictures(gl);
        self.generation += 1;
        for p in pictures {
            if p.rgba.len() != p.width as usize * p.height as usize * 4 {
                return Err(format!(
                    "picture {}x{} has {} bytes",
                    p.width,
                    p.height,
                    p.rgba.len()
                ));
            }
            self.pictures.push(upload_picture(gl, p)?);
        }
        Ok(())
    }

    /// Draw `draws` with `uniforms` set once, and the `passes` asked for,
    /// into `target`.
    ///
    /// # Errors
    ///
    /// An offscreen buffer the driver could not create or complete, a table
    /// that is not a whole number of rows, or a screen effect asked for with
    /// no screen programs.
    pub fn draw(
        &mut self,
        gl: &glow::Context,
        uniforms: &[(&'static str, Value)],
        draws: &[Draw],
        passes: &Passes<'_>,
        target: &Target,
    ) -> Result<Option<Vec<u8>>, String> {
        let [_, _, w, h] = target.viewport;
        if w <= 0 || h <= 0 {
            return Ok(None);
        }
        self.ensure_buffers(gl, [w, h])?;
        self.fit_inputs(gl, passes)?;
        self.fill_layers(gl, passes.layers, draws)?;
        let Some(multisampled) = self.buffers.as_ref().map(|b| b.multisampled) else {
            return Ok(None);
        };
        self.begin(gl, multisampled, [w, h], target.background);
        for (name, value) in uniforms {
            self.main.set(gl, name, value);
        }
        self.bind_inputs(gl, passes);
        self.draw_runs(gl, draws, None);
        if let Some(effect) = &passes.screen {
            self.unbind_inputs(gl, passes);
            self.screen(gl, effect, [w, h])?;
        }
        if let Some(repeat) = &passes.repeat {
            self.repeat(gl, multisampled, [w, h], passes, repeat, draws);
        }
        overlay::draw_all(gl, &mut self.overlays, passes.overlays);
        self.unbind_inputs(gl, passes);
        let Some(b) = &self.buffers else {
            return Ok(None);
        };
        b.resolve(gl);
        Ok(buffers::present(gl, b, target))
    }

    /// Make the tables and layered textures match `passes`, keeping what
    /// still fits.
    fn fit_inputs(&mut self, gl: &glow::Context, passes: &Passes<'_>) -> Result<(), String> {
        let mut old = std::mem::take(&mut self.tables).into_iter();
        for spec in passes.tables {
            self.tables.push(TableTexture::fit(gl, old.next(), spec)?);
        }
        old.for_each(|t| t.delete(gl));
        let mut old = std::mem::take(&mut self.layer_targets).into_iter();
        for spec in passes.layers {
            self.layer_targets
                .push(LayerTarget::fit(gl, old.next(), spec)?);
        }
        old.for_each(|t| t.delete(gl));
        Ok(())
    }

    /// Fill each layered texture asked for or out of date.
    fn fill_layers(
        &mut self,
        gl: &glow::Context,
        specs: &[Layers<'_>],
        draws: &[Draw],
    ) -> Result<(), String> {
        let generation = self.generation;
        let mut bound = false;
        for (target, spec) in self.layer_targets.iter_mut().zip(specs) {
            if !target.wants_fill(spec, generation) {
                continue;
            }
            if !bound {
                bound = true;
                // SAFETY: the vertex array is this mesh's own.
                unsafe { gl.bind_vertex_array(Some(self.vao)) };
            }
            target.fill(
                gl,
                spec,
                &mut self.programs,
                draws,
                &self.pictures,
                generation,
            )?;
        }
        Ok(())
    }

    /// Bind every table and layered texture to its unit and sampler on the
    /// main program, which must be in use.
    fn bind_inputs(&mut self, gl: &glow::Context, passes: &Passes<'_>) {
        for (t, spec) in self.tables.iter().zip(passes.tables) {
            t.bind(gl, &mut self.main, spec);
        }
        for (t, spec) in self.layer_targets.iter().zip(passes.layers) {
            t.bind(gl, &mut self.main, spec);
        }
        // SAFETY: a binding change on the caller's current context.
        unsafe { gl.active_texture(glow::TEXTURE0) };
    }

    /// Unbind what [`Mesh::bind_inputs`] bound.
    fn unbind_inputs(&self, gl: &glow::Context, passes: &Passes<'_>) {
        for spec in passes.tables {
            // SAFETY: a binding change on the caller's current context.
            unsafe {
                gl.active_texture(glow::TEXTURE0 + spec.unit);
                gl.bind_texture(glow::TEXTURE_2D, None);
            }
        }
        for (t, spec) in self.layer_targets.iter().zip(passes.layers) {
            t.unbind(gl, spec.unit);
        }
        // SAFETY: a binding change on the caller's current context.
        unsafe { gl.active_texture(glow::TEXTURE0) };
    }

    /// Run the screen effect over the faces in the multisampled buffer.
    fn screen(
        &mut self,
        gl: &glow::Context,
        effect: &ScreenEffect<'_>,
        size: [i32; 2],
    ) -> Result<(), String> {
        if self.screen_textures.as_ref().is_none_or(|t| t.size != size) {
            if let Some(old) = self.screen_textures.take() {
                old.delete(gl);
            }
            self.screen_textures = Some(ScreenTextures::new(gl, size)?);
        }
        let (Some(programs), Some(b), Some(t)) = (
            self.screen_programs.as_mut(),
            &self.buffers,
            &self.screen_textures,
        ) else {
            return Err("screen effect asked for with no screen programs".to_owned());
        };
        programs.run(gl, effect, b, t);
        Ok(())
    }

    /// Draw the opaque draws again with `repeat`'s uniforms, added in where
    /// they meet the faces' depth exactly.
    fn repeat(
        &mut self,
        gl: &glow::Context,
        multisampled: glow::Framebuffer,
        [w, h]: [i32; 2],
        passes: &Passes<'_>,
        repeat: &Repeat<'_>,
        draws: &[Draw],
    ) {
        let [r, g, b, a] = repeat.blend;
        // SAFETY: the framebuffer, program and vertex array are this mesh's own.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(multisampled));
            gl.viewport(0, 0, w, h);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::EQUAL);
            gl.depth_mask(false);
            gl.use_program(Some(self.main.id));
            gl.bind_vertex_array(Some(self.vao));
        }
        self.bind_inputs(gl, passes);
        // SAFETY: state changes on the caller's current context.
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_equation(glow::FUNC_ADD);
            gl.blend_color(r, g, b, a);
            gl.blend_func(glow::CONSTANT_COLOR, glow::ONE);
        }
        self.draw_runs(gl, draws, Some(repeat.uniforms));
        // SAFETY: state changes on the caller's current context.
        unsafe { gl.depth_func(glow::LESS) };
    }

    /// Bind the multisampled buffer, clear it to `background`, and set the
    /// state the opaque draws need.
    fn begin(
        &self,
        gl: &glow::Context,
        multisampled: glow::Framebuffer,
        [w, h]: [i32; 2],
        background: [f32; 4],
    ) {
        // SAFETY: the framebuffer is this mesh's own and complete; the
        // program and vertex array are this mesh's own.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(multisampled));
            gl.viewport(0, 0, w, h);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::BLEND);
            gl.disable(glow::CULL_FACE);
            gl.depth_mask(true);
            gl.color_mask(true, true, true, true);
            let [r, g, b, a] = background;
            gl.clear_color(r, g, b, a);
            gl.clear_depth_f32(1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.use_program(Some(self.main.id));
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    /// The draws, opaque then blended, each with its uniforms and picture.
    /// With `repeat`, only the opaque ones, `repeat` set after each's own,
    /// under the blending the caller set.
    fn draw_runs(
        &mut self,
        gl: &glow::Context,
        draws: &[Draw],
        repeat: Option<&[(&'static str, Value)]>,
    ) {
        let mut blending = false;
        for d in draws {
            if d.translucent && repeat.is_some() {
                break;
            }
            if d.translucent && !blending {
                blending = true;
                // SAFETY: state changes on the caller's current context.
                unsafe {
                    gl.enable(glow::BLEND);
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func_separate(
                        glow::SRC_ALPHA,
                        glow::ONE_MINUS_SRC_ALPHA,
                        glow::ONE,
                        glow::ONE_MINUS_SRC_ALPHA,
                    );
                    gl.depth_mask(false);
                }
            }
            for (name, value) in d.uniforms.iter().chain(repeat.unwrap_or_default()) {
                self.main.set(gl, name, value);
            }
            let texture = d.picture.and_then(|i| self.pictures.get(i).copied());
            // SAFETY: the texture is this mesh's own or none; the range is
            // the caller's and GL ignores vertices past the buffer's end
            // only by raising an error, never by reading out of bounds.
            unsafe {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, texture);
                gl.draw_arrays(glow::TRIANGLES, d.first, d.count);
            }
        }
        // SAFETY: restores the opaque state and unbinds this mesh's texture.
        unsafe {
            gl.disable(glow::BLEND);
            gl.depth_mask(true);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Make the offscreen buffers `size` pixels, keeping them when they are.
    fn ensure_buffers(&mut self, gl: &glow::Context, size: [i32; 2]) -> Result<(), String> {
        if self.buffers.as_ref().is_some_and(|b| b.size == size) {
            return Ok(());
        }
        if let Some(old) = self.buffers.take() {
            old.delete(gl);
        }
        self.buffers = Some(Buffers::new(gl, size, self.samples)?);
        Ok(())
    }

    /// Delete the pictures.
    fn drop_pictures(&mut self, gl: &glow::Context) {
        for texture in self.pictures.drain(..) {
            // SAFETY: the texture is this mesh's own.
            unsafe { gl.delete_texture(texture) };
        }
    }

    /// Delete every GL object this mesh owns.
    pub fn destroy(mut self, gl: &glow::Context) {
        self.drop_pictures(gl);
        if let Some(b) = self.buffers.take() {
            b.delete(gl);
        }
        if let Some(t) = self.screen_textures.take() {
            t.delete(gl);
        }
        if let Some(p) = self.screen_programs.take() {
            p.delete(gl);
        }
        self.layer_targets.drain(..).for_each(|t| t.delete(gl));
        self.tables.drain(..).for_each(|t| t.delete(gl));
        self.programs.drain(..).for_each(|p| p.delete(gl));
        self.overlays.drain(..).for_each(|o| o.delete(gl));
        // SAFETY: these objects are this mesh's own.
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
        self.main.delete(gl);
    }
}

/// Upload `p` as a linear-filtered `RGBA8` texture with its wrap modes.
fn upload_picture(gl: &glow::Context, p: &Picture) -> Result<glow::Texture, String> {
    // SAFETY: the texture is created here and returned to the caller.
    unsafe {
        let texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        for (axis, wrap) in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
            .into_iter()
            .zip(p.wrap)
        {
            let mode = match wrap {
                Wrap::Repeat => glow::REPEAT,
                Wrap::Clamp => glow::CLAMP_TO_EDGE,
                Wrap::Mirrored => glow::MIRRORED_REPEAT,
            };
            gl.tex_parameter_i32(glow::TEXTURE_2D, axis, mode as i32);
        }
        for filter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, filter, glow::LINEAR as i32);
        }
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            p.width as i32,
            p.height as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(p.rgba)),
        );
        gl.bind_texture(glow::TEXTURE_2D, None);
        Ok(texture)
    }
}
