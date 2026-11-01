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
//! State left behind: the destination framebuffer bound, no program, no
//! vertex array, depth test, blending and culling off, scissor on with the
//! clip box. `egui_glow` re-prepares its own state after a callback.

use std::collections::HashMap;

use glow::HasContext as _;

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
    /// `int` or `bool`.
    Int(i32),
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

/// The offscreen buffers a frame is drawn into.
struct Buffers {
    size: [i32; 2],
    multisampled: glow::Framebuffer,
    ms_colour: glow::Renderbuffer,
    ms_depth: glow::Renderbuffer,
    resolved: glow::Framebuffer,
    colour: glow::Renderbuffer,
}

/// A compiled program, its geometry and pictures, and its buffers.
pub struct Mesh {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    pictures: Vec<glow::Texture>,
    locations: HashMap<&'static str, Option<glow::UniformLocation>>,
    buffers: Option<Buffers>,
    samples: i32,
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
        // SAFETY: every call is on the caller's current context; objects
        // created here are owned by the returned `Mesh` or deleted on error.
        unsafe {
            let program = link(gl, vertex, fragment)?;
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
                program,
                vao,
                vbo,
                pictures: Vec::new(),
                locations: HashMap::new(),
                buffers: None,
                samples,
            })
        }
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

    /// Draw `draws` with `uniforms` set once, into `target`.
    ///
    /// # Errors
    ///
    /// An offscreen buffer the driver could not create or complete.
    pub fn draw(
        &mut self,
        gl: &glow::Context,
        uniforms: &[(&'static str, Value)],
        draws: &[Draw],
        target: &Target,
    ) -> Result<Option<Vec<u8>>, String> {
        let [_, _, w, h] = target.viewport;
        if w <= 0 || h <= 0 {
            return Ok(None);
        }
        self.ensure_buffers(gl, [w, h])?;
        let Some(b) = &self.buffers else {
            return Ok(None);
        };
        let (multisampled, resolved) = (b.multisampled, b.resolved);
        self.begin(gl, multisampled, [w, h], target.background);
        for (name, value) in uniforms {
            self.set(gl, name, value);
        }
        self.draw_runs(gl, draws);
        Ok(present(gl, [multisampled, resolved], [w, h], target))
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
            gl.use_program(Some(self.program));
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
                self.set(gl, name, value);
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

    /// Set the uniform `name`, when the linked program has it.
    fn set(&mut self, gl: &glow::Context, name: &'static str, value: &Value) {
        let program = self.program;
        let location = self
            .locations
            .entry(name)
            // SAFETY: a query on this mesh's own linked program.
            .or_insert_with(|| unsafe { gl.get_uniform_location(program, name) })
            .as_ref();
        if location.is_none() {
            return;
        }
        // SAFETY: the location belongs to the program in use.
        unsafe {
            match value {
                Value::Int(v) => gl.uniform_1_i32(location, *v),
                Value::Vec3(v) => gl.uniform_3_f32_slice(location, v),
                Value::Vec4(v) => gl.uniform_4_f32_slice(location, v),
                Value::Mat4(v) => gl.uniform_matrix_4_f32_slice(location, false, v),
                Value::Vec3s(v) => gl.uniform_3_f32_slice(location, v),
                Value::Vec4s(v) => gl.uniform_4_f32_slice(location, v),
            }
        }
    }

    /// Make the offscreen buffers `size` pixels, keeping them when they are.
    fn ensure_buffers(&mut self, gl: &glow::Context, size: [i32; 2]) -> Result<(), String> {
        if self.buffers.as_ref().is_some_and(|b| b.size == size) {
            return Ok(());
        }
        if let Some(old) = self.buffers.take() {
            delete_buffers(gl, &old);
        }
        let [w, h] = size;
        // SAFETY: objects created here are kept in `self.buffers` or
        // deleted before returning an error.
        unsafe {
            let ms_colour = gl.create_renderbuffer()?;
            let ms_depth = gl.create_renderbuffer()?;
            let colour = gl.create_renderbuffer()?;
            let multisampled = gl.create_framebuffer()?;
            let resolved = gl.create_framebuffer()?;
            let buffers = Buffers {
                size,
                multisampled,
                ms_colour,
                ms_depth,
                resolved,
                colour,
            };
            for (rb, samples, format) in [
                (ms_colour, self.samples, glow::RGBA8),
                (ms_depth, self.samples, glow::DEPTH_COMPONENT24),
                (colour, 0, glow::RGBA8),
            ] {
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rb));
                gl.renderbuffer_storage_multisample(glow::RENDERBUFFER, samples, format, w, h);
            }
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            let attach = [
                (multisampled, glow::COLOR_ATTACHMENT0, ms_colour),
                (multisampled, glow::DEPTH_ATTACHMENT, ms_depth),
                (resolved, glow::COLOR_ATTACHMENT0, colour),
            ];
            for (fb, point, rb) in attach {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, point, glow::RENDERBUFFER, Some(rb));
            }
            for fb in [multisampled, resolved] {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
                if status != glow::FRAMEBUFFER_COMPLETE {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                    delete_buffers(gl, &buffers);
                    return Err(format!("offscreen buffer {w}x{h} incomplete: {status:#x}"));
                }
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.buffers = Some(buffers);
        }
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
            delete_buffers(gl, &b);
        }
        // SAFETY: these objects are this mesh's own.
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
            gl.delete_program(self.program);
        }
    }
}

/// Resolve the multisampled buffer, read it back when asked, and copy it to
/// the target's viewport, clipped; leaves the destination bound.
fn present(
    gl: &glow::Context,
    [multisampled, resolved]: [glow::Framebuffer; 2],
    [w, h]: [i32; 2],
    target: &Target,
) -> Option<Vec<u8>> {
    // SAFETY: the framebuffers are the mesh's own and complete, and the
    // resolved one is bound for reading when `read` runs; the destination
    // is the caller's framebuffer.
    unsafe {
        gl.depth_mask(true);
        gl.disable(glow::DEPTH_TEST);
        gl.disable(glow::BLEND);
        gl.bind_vertex_array(None);
        gl.use_program(None);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(multisampled));
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(resolved));
        gl.blit_framebuffer(
            0,
            0,
            w,
            h,
            0,
            0,
            w,
            h,
            glow::COLOR_BUFFER_BIT,
            glow::NEAREST,
        );
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(resolved));
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

/// Delete one set of offscreen buffers.
fn delete_buffers(gl: &glow::Context, b: &Buffers) {
    // SAFETY: the objects are the caller's mesh's own.
    unsafe {
        gl.delete_framebuffer(b.multisampled);
        gl.delete_framebuffer(b.resolved);
        gl.delete_renderbuffer(b.ms_colour);
        gl.delete_renderbuffer(b.ms_depth);
        gl.delete_renderbuffer(b.colour);
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
