//! What a caller hands [`crate::Mesh::draw`]: the draws, the target, and the
//! passes around the main one. Plain data; nothing here touches GL.

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
    /// `float[n]`.
    Floats(Vec<f32>),
    /// `int[n]` or `bool[n]`.
    Ints(Vec<i32>),
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
    /// [`crate::Mesh::set_pictures`].
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

/// A table of floats the main program reads with `texelFetch`: an `RGBA32F`
/// texture `width` texels across, `NEAREST`, no mipmaps. Re-uploaded only
/// when its contents change.
#[derive(Debug, Clone, Copy)]
pub struct Table<'a> {
    /// The main program's `sampler2D`.
    pub sampler: &'static str,
    /// The texture unit, 1–15, used by no other input.
    pub unit: u32,
    /// Texels a row.
    pub width: i32,
    /// Four floats a texel, row 0 first; a whole number of rows, at least one.
    pub data: &'a [f32],
}

/// What a [`Layers`] texture holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 32-bit float depth: fills write depth, test `LESS`, no colour.
    Depth,
    /// One 8-bit channel cleared to 1: fills multiply it by their colour
    /// (`ZERO`, `SRC_COLOR`), depth test off.
    Coverage,
}

/// Which draws a [`Fill`] draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The opaque draws.
    Opaque,
    /// The translucent draws, each with its own uniforms and picture.
    Translucent,
}

/// One program's pass over some draws into one layer.
#[derive(Debug, Clone, Copy)]
pub struct Fill<'a> {
    /// The layer, 0 for a plain texture.
    pub layer: i32,
    /// A program from [`crate::Mesh::add_program`].
    pub program: usize,
    /// Set on the program before the draws.
    pub uniforms: &'a [(&'static str, Value)],
    /// The draws.
    pub source: Source,
}

/// A square texture, or array of them, filled from other viewpoints before
/// the main pass and bound to the main program as a sampler. Every layer is
/// cleared before a redraw, so a layer no fill names reads as cleared.
#[derive(Debug, Clone, Copy)]
pub struct Layers<'a> {
    /// The main program's sampler.
    pub sampler: &'static str,
    /// The texture unit, 1–15, used by no other input.
    pub unit: u32,
    /// A `TEXTURE_2D_ARRAY`; else a `TEXTURE_2D` of one layer.
    pub array: bool,
    /// What it holds.
    pub format: Format,
    /// Texels a side.
    pub size: i32,
    /// How many layers; 1 when not an array.
    pub layers: i32,
    /// In order.
    pub fills: &'a [Fill<'a>],
    /// Fill it this frame. It is also filled when new, reshaped, or the
    /// vertices or pictures changed since; otherwise it is reused.
    pub redraw: bool,
}

/// The opaque draws again over the finished faces, with `uniforms` set
/// after each draw's own: depth `EQUAL`, no depth write, and the result
/// added in as `blend × colour` (`CONSTANT_COLOR`, `ONE`).
#[derive(Debug, Clone, Copy)]
pub struct Repeat<'a> {
    /// Set on the main program after each draw's uniforms.
    pub uniforms: &'a [(&'static str, Value)],
    /// The blend constant, RGBA.
    pub blend: [f32; 4],
}

/// One vertex attribute of an overlay: shader location, floats, and byte
/// offset into a vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute {
    /// The shader location.
    pub location: u32,
    /// Floats, 1–4.
    pub size: i32,
    /// Bytes from the start of a vertex.
    pub offset: i32,
}

/// How an overlay's vertices are laid out.
#[derive(Debug, Clone, Copy)]
pub struct Layout<'a> {
    /// Bytes from one vertex (or instance) to the next.
    pub stride: i32,
    /// 1 to advance per instance, 0 per vertex.
    pub divisor: u32,
    /// Every attribute read.
    pub attributes: &'a [Attribute],
}

/// What an overlay draw emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    /// A `vertices`-vertex triangle strip per instance.
    InstancedStrip {
        /// Vertices a strip.
        vertices: i32,
        /// How many strips.
        instances: i32,
    },
    /// `count` points, sized by the vertex shader.
    Points {
        /// How many.
        count: i32,
    },
}

/// One overlay drawn over the faces after the screen effect and the
/// [`Repeat`]: depth `LEQUAL` with depth write, source-alpha blending.
#[derive(Debug, Clone, Copy)]
pub struct Overlay<'a> {
    /// An overlay from [`crate::Mesh::add_overlay`].
    pub overlay: usize,
    /// Set on its program.
    pub uniforms: &'a [(&'static str, Value)],
    /// What it draws.
    pub primitive: Primitive,
}

/// A two-pass effect over the finished faces, with the programs
/// [`crate::Mesh::set_screen_programs`] linked; each draws one triangle
/// covering the viewport. The first reads the frame's depth and writes a
/// texture the size of the viewport; the second reads the frame's colour
/// and depth and that texture, and writes the frame's colour, leaving its
/// depth for what follows.
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

/// The optional passes around the main one, in drawing order: tables and
/// layered textures before it, then the screen effect, the repeat and the
/// overlays after it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passes<'a> {
    /// Bound for the main pass and the repeat.
    pub tables: &'a [Table<'a>],
    /// Filled before the main pass and bound for it.
    pub layers: &'a [Layers<'a>],
    /// Drawn over the main pass.
    pub screen: Option<ScreenEffect<'a>>,
    /// Drawn after the screen effect.
    pub repeat: Option<Repeat<'a>>,
    /// Drawn last.
    pub overlays: &'a [Overlay<'a>],
    /// Drawn before the main pass and bound for it and the repeat.
    pub prepass: Option<Prepass<'a>>,
    /// Draws the frame as a fisheye instead of one perspective frame.
    pub lens: Option<Lens<'a>>,
}

/// A fisheye frame: each face drawn as an ordinary frame at its own size,
/// single-sample, then resampled into the picture by the lens program
/// ([`crate::Mesh::set_lens_program`]) with the depth test on, `ALWAYS`,
/// and depth writes on; the overlays follow, into the picture. With one,
/// [`Passes::screen`], [`Passes::repeat`] and [`Passes::prepass`] are not
/// drawn: each face carries its own screen effect and repeat.
#[derive(Debug, Clone, Copy)]
pub struct Lens<'a> {
    /// Drawn in order; each resample writes only the pixels its face owns.
    pub faces: &'a [LensFace<'a>],
    /// Set on the lens program for every face.
    pub uniforms: &'a [(&'static str, Value)],
    /// The lens program's sampler for a face's colour, on unit 0.
    pub colour: &'static str,
    /// Its sampler for a face's 24-bit depth, on unit 1.
    pub depth: &'static str,
}

/// One face of a [`Lens`].
#[derive(Debug, Clone, Copy)]
pub struct LensFace<'a> {
    /// Its width and height, pixels.
    pub size: [i32; 2],
    /// Set on the main program before the face's draws; every key the
    /// repeat overrides must be among them.
    pub uniforms: &'a [(&'static str, Value)],
    /// Drawn over the face's draws.
    pub screen: Option<ScreenEffect<'a>>,
    /// Drawn after its screen effect.
    pub repeat: Option<Repeat<'a>>,
    /// Set on the lens program after [`Lens::uniforms`] for this face.
    pub resample: &'a [(&'static str, Value)],
}

/// A depth-only pass over the opaque draws before the main one, into a
/// single-sample depth texture the main program samples by window pixel.
#[derive(Debug, Clone, Copy)]
pub struct Prepass<'a> {
    /// The [`crate::Mesh::add_program`] program drawn with.
    pub program: usize,
    /// Set on that program.
    pub uniforms: &'a [(&'static str, Value)],
    /// The main program's sampler the depth texture is bound to.
    pub sampler: &'static str,
    /// Its texture unit.
    pub unit: u32,
}
