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
//! Two optional passes surround the main one ([`Passes`]): a square depth
//! map drawn first with its own program and bound to the main program as a
//! sampler, and a two-pass effect over the finished frame's colour and depth.
//!
//! State left behind: the destination framebuffer bound, no program, no
//! vertex array, depth test, blending and culling off, scissor on with the
//! clip box. `egui_glow` re-prepares its own state after a callback.

mod buffers;
mod passes;
mod program;

use glow::HasContext as _;

use buffers::{Buffers, ScreenTextures};
use passes::{DepthTarget, ScreenPrograms};
use program::Program;

/// How a picture's coordinates outside 0–1 are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    /// The picture tiles.
    Repeat,
    /// Coordinates are held to the edge.
    Clamp,
    /// The picture tiles, every other copy mirrored.
    Mirrored,
}

/// A picture to sample: straight RGBA, top row first, four bytes a pixel.
#[derive(Debug, Clone, Copy)]
pub struct Picture<'a> {
    /// Pixels across.
    pub width: u32,
    /// Pixels down.
    pub height: u32,
    /// `width × height × 4` bytes.
    pub rgba: &'a [u8],
    /// Across, then down.
    pub wrap: [Wrap; 2],
}

/// A uniform's value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `int`, `bool` or a sampler's texture unit.
    Int(i32),
    /// `float`.
    Float(f32),
    /// `ivec2`.
    IVec2([i32; 2]),
    /// `vec2`.
    Vec2([f32; 2]),
    /// `vec3`.
    Vec3([f32; 3]),
    /// `vec4`.
    Vec4([f32; 4]),
    /// `mat4`, column-major.
    Mat4([f32; 16]),
    /// `vec3[n]`, three floats an entry.
    Vec3s(Vec<f32>),
    /// `vec4[n]`, four floats an entry.
    Vec4s(Vec<f32>),
}

/// One draw: a run of whole triangles with its own uniforms.
#[derive(Debug, Clone, PartialEq)]
pub struct Draw {
    /// The first vertex.
    pub first: i32,
    /// How many vertices, a multiple of three.
    pub count: i32,
    /// The picture bound to texture unit 0, an index into the last
    /// [`Mesh::set_pictures`].
    pub picture: Option<usize>,
    /// Blended over what is drawn, without writing depth. Translucent draws
    /// must come after every opaque one.
    pub translucent: bool,
    /// Set before this draw.
    pub uniforms: Vec<(&'static str, Value)>,
}

/// Where a frame goes, in the destination's pixels from its bottom left.
#[derive(Debug, Clone, Copy)]
pub struct Target {
    /// `[left, bottom, width, height]` of the picture.
    pub viewport: [i32; 4],
    /// `[left, bottom, width, height]` the copy may touch.
    pub clip: [i32; 4],
    /// The framebuffer the callback runs in; `None` is the window's.
    pub destination: Option<glow::Framebuffer>,
    /// The clear colour, straight RGBA 0–1.
    pub background: [f32; 4],
    /// Also return the picture's pixels.
    pub read_back: bool,
}

/// A depth map drawn before the frame, from the viewpoint its uniforms set,
/// with the program [`Mesh::set_depth_program`] linked.
#[derive(Debug, Clone, Copy)]
pub struct DepthMap<'a> {
    /// Pixels a side.
    pub size: i32,
    /// Set on the depth map's program.
    pub uniforms: &'a [(&'static str, Value)],
    /// The main program's sampler the map is bound to.
    pub sampler: &'static str,
    /// Draw it this frame. A map is also drawn when it is new, resized, or
    /// the vertices changed since it was last drawn; otherwise it is reused.
    pub redraw: bool,
}

/// A two-pass effect over the finished frame, with the programs
/// [`Mesh::set_screen_programs`] linked; each draws one triangle covering
/// the viewport. The first reads the frame's depth and writes a texture the
/// size of the viewport; the second reads the frame's colour and depth and
/// that texture, and writes the picture.
#[derive(Debug, Clone, Copy)]
pub struct ScreenEffect<'a> {
    /// Set on both programs.
    pub uniforms: &'a [(&'static str, Value)],
    /// Both programs' sampler for the frame's depth, 24 bits, read `NEAREST`.
    pub depth: &'static str,
    /// The second program's sampler for the frame's colour.
    pub colour: &'static str,
    /// The second program's sampler for the first's output.
    pub first_output: &'static str,
}

/// The optional passes around the main one.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passes<'a> {
    /// Drawn before the main pass.
    pub depth_map: Option<DepthMap<'a>>,
    /// Drawn over the main pass.
    pub screen: Option<ScreenEffect<'a>>,
}

/// A compiled program, its geometry and pictures, and its buffers.
pub struct Mesh {
    main: Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    pictures: Vec<glow::Texture>,
    buffers: Option<Buffers>,
    samples: i32,
    depth_program: Option<Program>,
    depth_target: Option<DepthTarget>,
    /// The vertices changed since the depth map was drawn.
    depth_stale: bool,
    screen_programs: Option<ScreenPrograms>,
    screen_textures: Option<ScreenTextures>,
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
                depth_program: None,
                depth_target: None,
                depth_stale: true,
                screen_programs: None,
                screen_textures: None,
            })
        }
    }

    /// Link the program a [`DepthMap`] is drawn with; it reads the same
    /// vertex attributes as the main program.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log.
    pub fn set_depth_program(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
    ) -> Result<(), String> {
        let program = Program::new(gl, vertex, fragment)?;
        if let Some(old) = self.depth_program.replace(program) {
            old.delete(gl);
        }
        Ok(())
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
        self.depth_stale = true;
    }

    /// Replace the pictures [`Draw::picture`] indexes.
    ///
    /// # Errors
    ///
    /// A texture the driver could not create, or a picture whose bytes do
    /// not match its size.
    pub fn set_pictures(&mut self, gl: &glow::Context, pictures: &[Picture]) -> Result<(), String> {
        self.drop_pictures(gl);
        for p in pictures {
            if p.rgba.len() != p.width as usize * p.height as usize * 4 {
                return Err(format!(
                    "picture {}x{} has {} bytes",
                    p.width,
                    p.height,
                    p.rgba.len()
                ));
            }
            // SAFETY: the texture is created here and kept in `self.pictures`.
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
                self.pictures.push(texture);
            }
        }
        Ok(())
    }

    /// Draw `draws` with `uniforms` set once, and the `passes` asked for,
    /// into `target`.
    ///
    /// # Errors
    ///
    /// An offscreen buffer the driver could not create or complete, or a
    /// pass asked for whose program is not linked.
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
        if let Some(map) = &passes.depth_map {
            self.depth_pass(gl, map, draws)?;
        }
        let Some(b) = &self.buffers else {
            return Ok(None);
        };
        self.begin(gl, b.multisampled, [w, h], target.background);
        for (name, value) in uniforms {
            self.main.set(gl, name, value);
        }
        if let (Some(map), Some(t)) = (&passes.depth_map, &self.depth_target) {
            t.bind(gl, &mut self.main, map.sampler);
        }
        self.draw_runs(gl, draws);
        self.finish(gl, passes.screen.as_ref(), [w, h])?;
        let Some(b) = &self.buffers else {
            return Ok(None);
        };
        Ok(buffers::present(gl, b, target))
    }

    /// Draw the depth map when it is asked for or out of date.
    fn depth_pass(
        &mut self,
        gl: &glow::Context,
        map: &DepthMap<'_>,
        draws: &[Draw],
    ) -> Result<(), String> {
        let Some(program) = self.depth_program.as_mut() else {
            return Err("depth map asked for with no depth program".to_owned());
        };
        let size = map.size.max(1);
        let fresh = self.depth_target.as_ref().is_none_or(|t| t.size != size);
        if fresh {
            if let Some(old) = self.depth_target.take() {
                old.delete(gl);
            }
            self.depth_target = Some(DepthTarget::new(gl, size)?);
        }
        if !(fresh || map.redraw || self.depth_stale) {
            return Ok(());
        }
        let Some(target) = &self.depth_target else {
            return Ok(());
        };
        // SAFETY: the vertex array is this mesh's own.
        unsafe { gl.bind_vertex_array(Some(self.vao)) };
        program.use_with(gl, map.uniforms);
        target.draw(gl, draws);
        self.depth_stale = false;
        Ok(())
    }

    /// Resolve the frame into the plain buffer, through the screen effect
    /// when one is asked for.
    fn finish(
        &mut self,
        gl: &glow::Context,
        screen: Option<&ScreenEffect<'_>>,
        size: [i32; 2],
    ) -> Result<(), String> {
        // SAFETY: unbinds the depth map's unit.
        unsafe {
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.active_texture(glow::TEXTURE0);
        }
        let Some(effect) = screen else {
            if let Some(b) = &self.buffers {
                b.resolve(gl);
            }
            return Ok(());
        };
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
    fn draw_runs(&mut self, gl: &glow::Context, draws: &[Draw]) {
        let mut blending = false;
        for d in draws {
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
            for (name, value) in &d.uniforms {
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
        // SAFETY: unbinds this mesh's texture.
        unsafe { gl.bind_texture(glow::TEXTURE_2D, None) };
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
        if let Some(t) = self.depth_target.take() {
            t.delete(gl);
        }
        if let Some(t) = self.screen_textures.take() {
            t.delete(gl);
        }
        if let Some(p) = self.depth_program.take() {
            p.delete(gl);
        }
        if let Some(p) = self.screen_programs.take() {
            p.delete(gl);
        }
        // SAFETY: these objects are this mesh's own.
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
        self.main.delete(gl);
    }
}
