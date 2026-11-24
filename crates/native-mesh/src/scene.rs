//! A caller's one-triangle program drawn over an `RGBA32I` scene texture into
//! `RGBA8` framebuffers, tile by tile, and the pixels read back.
//!
//! Contract: a [`SceneDrawer`] is made and used on the context the caller
//! paints with, and [`SceneDrawer::destroy`]ed on it. The program reads the
//! scene through the sampler `u_scene` on unit 0 and is told each tile's
//! first pixel through the `ivec2` uniform `u_origin`, so framebuffer pixel
//! `(x, y)` is output pixel `u_origin + (x, y)`. Row `y` of each read tile is
//! taken as output row `u_origin.y + y`: the program, not this module, decides
//! which way up the picture is.
//!
//! A picture is drawn as a [`SceneJob`]. [`SceneDrawer::begin`] allocates the
//! scene texture; each [`SceneDrawer::step`] first uploads up to
//! [`UPLOAD_BYTES`] of it, then, once it is all on the card, submits one slice
//! of rows. Every step returns at once, so a caller on the UI thread spreads a page over many frames and
//! never waits on the card. Each slice is fenced, and at most [`QUEUED`] are
//! on the card at once: enough that it never idles between frames, few enough
//! that the caller's own frame is not queued behind much. A slice's height
//! doubles when the card finished it by the next step and halves when it took
//! three or more. Tiles are read into one pixel-pack buffer, so the read-back
//! is asynchronous too, and the picture is copied out once, when its last
//! fence signals. [`SceneDrawer::draw`] runs the same job to the end with
//! blocking waits.
//!
//! State left behind by every call: no framebuffer, program, vertex array,
//! buffer or texture bound, scissor and blending off, pack row length 0.
//! `egui_glow` re-prepares its own state each frame.

use glow::HasContext as _;

use crate::Value;
use crate::program::Program;

/// Rows one draw call covers, so one submission stays short on a slow or
/// software GPU.
pub const BAND_ROWS: u32 = 256;

/// Scene bytes one step uploads: a 134 MB scene sent whole holds the caller's
/// frame for about 200 ms.
pub const UPLOAD_BYTES: usize = 8 << 20;

/// Slices on the card at once.
pub const QUEUED: usize = 2;

/// The first slice's height in rows, and the least it halves to.
const FIRST_SLICE: i32 = 64;
const MIN_SLICE: i32 = 16;

/// Largest tile edge this module asks for, below the driver's own limits.
const TILE_CAP: i32 = 4096;

/// The integer texture a scene is packed into: `width` × `height` × `layers`
/// texels of four `i32`s, layer 0 row 0 first.
#[derive(Debug, Clone, Copy)]
pub struct SceneTexture<'a> {
    /// Texels a row.
    pub width: u32,
    /// Rows a layer.
    pub height: u32,
    /// Layers, at least 1.
    pub layers: u32,
    /// The values, four a texel.
    pub texels: &'a [i32],
}

/// The program and the empty vertex array its triangle is drawn with.
pub struct SceneDrawer {
    program: Program,
    vao: glow::VertexArray,
}

/// Where a [`SceneDrawer::step`] left the job.
#[derive(Debug)]
pub enum Step {
    /// More to do; step again on a later frame.
    Pending,
    /// The picture's `RGBA8` bytes, row 0 first. The job is spent.
    Done(Vec<u8>),
}

/// One picture being drawn: its GL objects and how far it has got. Release it
/// with [`SceneJob::destroy`] on the context it was begun on, finished or not.
pub struct SceneJob {
    texture: glow::Texture,
    /// The scene's `[width, height, layers]`, and the rows (layer-major) sent.
    shape: [u32; 3],
    uploaded: u32,
    target: Target,
    pack: glow::Buffer,
    size: [i32; 2],
    tile: i32,
    /// The current tile's first pixel, and the next row within it.
    at: [i32; 2],
    row: i32,
    slice: i32,
    /// The slices on the card, oldest first.
    fences: std::collections::VecDeque<glow::Fence>,
    /// Steps that found the oldest fence unsignalled.
    waited: u32,
    steps: u32,
}

impl SceneJob {
    /// Slices submitted so far.
    #[must_use]
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// Delete every GL object.
    pub fn destroy(self, gl: &glow::Context) {
        // SAFETY: the objects are this job's own.
        unsafe {
            for fence in self.fences {
                gl.delete_sync(fence);
            }
            gl.delete_buffer(self.pack);
            gl.delete_texture(self.texture);
        }
        self.target.destroy(gl);
    }
}

impl SceneDrawer {
    /// Compile `vertex` and `fragment`.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log, or a failed allocation.
    pub fn new(gl: &glow::Context, vertex: &str, fragment: &str) -> Result<Self, String> {
        let program = Program::new(gl, vertex, fragment)?;
        // SAFETY: on the caller's context; the array is this value's own.
        let vao = match unsafe { gl.create_vertex_array() } {
            Ok(v) => v,
            Err(e) => {
                program.delete(gl);
                return Err(e);
            }
        };
        Ok(Self { program, vao })
    }

    /// Draw `scene` into a `width` × `height` picture and return its `RGBA8`
    /// bytes, row 0 first, waiting on the card.
    ///
    /// # Errors
    ///
    /// As [`Self::begin`] and [`Self::step`].
    pub fn draw(
        &mut self,
        gl: &glow::Context,
        scene: SceneTexture<'_>,
        size: [u32; 2],
    ) -> Result<Vec<u8>, String> {
        let mut job = self.begin(gl, scene, size)?;
        let result = loop {
            match self.step(gl, &mut job, scene, true) {
                Ok(Step::Pending) => {}
                Ok(Step::Done(bytes)) => break Ok(bytes),
                Err(e) => break Err(e),
            }
        };
        job.destroy(gl);
        result
    }

    /// Upload `scene` and make what a `width` × `height` picture is drawn and
    /// read into; nothing is drawn yet.
    ///
    /// # Errors
    ///
    /// A texture the shape of which disagrees with its values, an empty or
    /// oversized picture, an incomplete framebuffer, or a failed allocation.
    pub fn begin(
        &mut self,
        gl: &glow::Context,
        scene: SceneTexture<'_>,
        [width, height]: [u32; 2],
    ) -> Result<SceneJob, String> {
        let want = scene.width as usize * scene.height as usize * scene.layers as usize * 4;
        if want == 0 || scene.texels.len() != want {
            return Err(format!(
                "a scene of {} values is not {want}",
                scene.texels.len()
            ));
        }
        let too_large = || format!("a {width}x{height} picture is too large to read back");
        let bytes =
            i32::try_from(u64::from(width) * u64::from(height) * 4).map_err(|_| too_large())?;
        if bytes == 0 {
            return Err("an empty picture".to_owned());
        }
        let (w, h) = (width as i32, height as i32);
        // SAFETY: a plain error-queue drain on the caller's context.
        unsafe { while gl.get_error() != glow::NO_ERROR {} }
        let tile = tile_edge(gl);
        let texture = allocate(gl, scene)?;
        let made =
            Target::new(gl, [tile.min(w), tile.min(h)]).and_then(|target| {
                match pack_buffer(gl, bytes) {
                    Ok(pack) => Ok((target, pack)),
                    Err(e) => {
                        target.destroy(gl);
                        Err(e)
                    }
                }
            });
        let (target, pack) = match made {
            Ok(m) => m,
            Err(e) => {
                // SAFETY: the texture is this call's own.
                unsafe { gl.delete_texture(texture) };
                return Err(e);
            }
        };
        Ok(SceneJob {
            texture,
            shape: [scene.width, scene.height, scene.layers],
            uploaded: 0,
            target,
            pack,
            size: [w, h],
            tile,
            at: [0, 0],
            row: 0,
            slice: FIRST_SLICE.min(tile),
            fences: std::collections::VecDeque::new(),
            waited: 0,
            steps: 0,
        })
    }

    /// Advance `job`: upload more of `scene`, which must be the one it was
    /// begun with, submit a slice, or collect the picture once the card has
    /// finished it. With `wait` false this never blocks: a slice the card has
    /// not finished returns [`Step::Pending`] at once.
    ///
    /// # Errors
    ///
    /// A GL error raised while drawing, a failed fence, or a read-back that
    /// would not map.
    pub fn step(
        &mut self,
        gl: &glow::Context,
        job: &mut SceneJob,
        scene: SceneTexture<'_>,
        wait: bool,
    ) -> Result<Step, String> {
        if [scene.width, scene.height, scene.layers] != job.shape {
            return Err("a step was given another scene".to_owned());
        }
        if !upload_some(gl, job, scene, if wait { usize::MAX } else { UPLOAD_BYTES })? {
            return Ok(Step::Pending);
        }
        while let Some(&fence) = job.fences.front() {
            let timeout = if wait { i32::MAX } else { 0 };
            // SAFETY: the fence is this job's own.
            let status =
                unsafe { gl.client_wait_sync(fence, glow::SYNC_FLUSH_COMMANDS_BIT, timeout) };
            match status {
                glow::TIMEOUT_EXPIRED => {
                    job.waited += 1;
                    break;
                }
                glow::WAIT_FAILED => return Err("the card's fence failed".to_owned()),
                _ => {}
            }
            // SAFETY: as above.
            unsafe { gl.delete_sync(fence) };
            job.fences.pop_front();
            job.slice = match job.waited {
                0 => (job.slice * 2).min(job.tile),
                1 | 2 => job.slice,
                _ => (job.slice / 2).max(MIN_SLICE),
            };
            job.waited = 0;
        }
        let drawn = job.at[1] >= job.size[1];
        if drawn && job.fences.is_empty() {
            return read_out(gl, job).map(Step::Done);
        }
        while !drawn && job.at[1] < job.size[1] && job.fences.len() < QUEUED {
            self.submit(gl, job)?;
        }
        Ok(Step::Pending)
    }

    /// Submit the next slice of `job`, queue its tile's read when the slice
    /// ends the tile, and fence the lot.
    fn submit(&mut self, gl: &glow::Context, job: &mut SceneJob) -> Result<(), String> {
        let [w, h] = job.size;
        let [x0, y0] = job.at;
        let (tw, th) = (job.tile.min(w - x0), job.tile.min(h - y0));
        let rows = job.slice.min(th - job.row);
        self.program.use_with(
            gl,
            &[
                ("u_scene", Value::Int(0)),
                ("u_origin", Value::IVec2([x0, y0])),
            ],
        );
        // SAFETY: every object named is this value's or the job's own, on
        // the caller's context; the state set here is reset below.
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(job.texture));
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(job.target.framebuffer));
            gl.bind_vertex_array(Some(self.vao));
            gl.disable(glow::BLEND);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::SCISSOR_TEST);
            gl.viewport(0, 0, tw, th);
            for band in (job.row..job.row + rows).step_by(BAND_ROWS as usize) {
                gl.scissor(0, band, tw, (BAND_ROWS as i32).min(job.row + rows - band));
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
            }
        }
        job.row += rows;
        job.steps += 1;
        if job.row >= th {
            read_tile(gl, job, [x0, y0], [tw, th]);
            job.row = 0;
            job.at = if x0 + tw < w {
                [x0 + tw, y0]
            } else {
                [0, y0 + th]
            };
        }
        // SAFETY: resets the state set above, on the caller's context.
        let error = unsafe {
            let error = gl.get_error();
            gl.disable(glow::SCISSOR_TEST);
            gl.bind_vertex_array(None);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
            gl.use_program(None);
            error
        };
        if error != glow::NO_ERROR {
            return Err(format!("GL error 0x{error:x} while drawing"));
        }
        // SAFETY: the fence is the job's own, deleted when it signals or when
        // the job is destroyed; the flush starts the slice before the next
        // frame.
        unsafe {
            job.fences
                .push_back(gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0)?);
            gl.flush();
        }
        Ok(())
    }

    /// Delete every GL object.
    pub fn destroy(self, gl: &glow::Context) {
        self.program.delete(gl);
        // SAFETY: the array is this value's own.
        unsafe { gl.delete_vertex_array(self.vao) };
    }
}

/// A `bytes`-long pixel-pack buffer, unbound.
fn pack_buffer(gl: &glow::Context, bytes: i32) -> Result<glow::Buffer, String> {
    // SAFETY: on the caller's context; the buffer is returned, and unbound.
    unsafe {
        let pack = gl.create_buffer()?;
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(pack));
        gl.buffer_data_size(glow::PIXEL_PACK_BUFFER, bytes, glow::STREAM_READ);
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
        Ok(pack)
    }
}

/// Queue the read of a finished `tw` × `th` tile at `(x0, y0)` into the job's
/// pack buffer, in place in the whole picture. The tile's framebuffer is
/// bound.
fn read_tile(gl: &glow::Context, job: &SceneJob, [x0, y0]: [i32; 2], [tw, th]: [i32; 2]) {
    let offset = (y0 as u32 * job.size[0] as u32 + x0 as u32) * 4;
    // SAFETY: the buffer is the job's own and holds the whole picture; the
    // row length lands each tile row on its picture row. Both reset here.
    unsafe {
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(job.pack));
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
        gl.pixel_store_i32(glow::PACK_ROW_LENGTH, job.size[0]);
        gl.read_pixels(
            0,
            0,
            tw,
            th,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::BufferOffset(offset),
        );
        gl.pixel_store_i32(glow::PACK_ROW_LENGTH, 0);
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
    }
}

/// Copy the finished picture out of the job's pack buffer.
fn read_out(gl: &glow::Context, job: &SceneJob) -> Result<Vec<u8>, String> {
    let len = job.size[0] as usize * job.size[1] as usize * 4;
    // SAFETY: the buffer is the job's own and `len` bytes long, and every
    // read into it has completed, its fence having signalled. It is unmapped
    // and unbound before return.
    unsafe {
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(job.pack));
        let mapped =
            gl.map_buffer_range(glow::PIXEL_PACK_BUFFER, 0, len as i32, glow::MAP_READ_BIT);
        let out = (!mapped.is_null()).then(|| {
            let bytes = std::slice::from_raw_parts(mapped, len).to_vec();
            gl.unmap_buffer(glow::PIXEL_PACK_BUFFER);
            bytes
        });
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
        out.ok_or_else(|| "the read-back would not map".to_owned())
    }
}

/// The tile edge: the smaller of the renderbuffer and viewport limits, capped.
fn tile_edge(gl: &glow::Context) -> i32 {
    // SAFETY: plain limit queries on the caller's context.
    let (renderbuffer, viewport) = unsafe {
        let mut dims = [0_i32; 2];
        gl.get_parameter_i32_slice(glow::MAX_VIEWPORT_DIMS, &mut dims);
        (
            gl.get_parameter_i32(glow::MAX_RENDERBUFFER_SIZE),
            dims[0].min(dims[1]),
        )
    };
    [renderbuffer, viewport]
        .into_iter()
        .filter(|v| *v > 0)
        .fold(TILE_CAP, i32::min)
}

/// Upload `scene` as an `RGBA32I` `TEXTURE_2D_ARRAY`.
fn allocate(gl: &glow::Context, scene: SceneTexture<'_>) -> Result<glow::Texture, String> {
    // SAFETY: on the caller's context; no data is read.
    unsafe {
        let texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
        crate::compute::nearest(gl, glow::TEXTURE_2D_ARRAY);
        gl.tex_image_3d(
            glow::TEXTURE_2D_ARRAY,
            0,
            glow::RGBA32I as i32,
            scene.width as i32,
            scene.height as i32,
            scene.layers as i32,
            0,
            glow::RGBA_INTEGER,
            glow::INT,
            glow::PixelUnpackData::Slice(None),
        );
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
        let error = gl.get_error();
        if error != glow::NO_ERROR {
            gl.delete_texture(texture);
            return Err(format!("GL error 0x{error:x} allocating the scene"));
        }
        Ok(texture)
    }
}

/// Send up to `budget` bytes more of `scene` (at least one row); whether it
/// is now all on the card.
fn upload_some(
    gl: &glow::Context,
    job: &mut SceneJob,
    scene: SceneTexture<'_>,
    budget: usize,
) -> Result<bool, String> {
    let [width, height, layers] = job.shape;
    let total = height * layers;
    if job.uploaded >= total {
        return Ok(true);
    }
    let row_values = width as usize * 4;
    let mut rows_left = (budget / (row_values * 4)).max(1);
    // SAFETY: on the caller's context; each slice is whole rows of `texels`,
    // the `i32`s viewed as their native-endian bytes, which is what `INT`
    // texels are.
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(job.texture));
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
        while rows_left > 0 && job.uploaded < total {
            let (layer, row) = (job.uploaded / height, job.uploaded % height);
            let n = (height - row).min(u32::try_from(rows_left).unwrap_or(u32::MAX));
            let first = job.uploaded as usize * row_values;
            let values = &scene.texels[first..first + n as usize * row_values];
            let bytes = std::slice::from_raw_parts(
                values.as_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            );
            gl.tex_sub_image_3d(
                glow::TEXTURE_2D_ARRAY,
                0,
                0,
                row as i32,
                layer as i32,
                width as i32,
                n as i32,
                1,
                glow::RGBA_INTEGER,
                glow::INT,
                glow::PixelUnpackData::Slice(Some(bytes)),
            );
            job.uploaded += n;
            rows_left -= n as usize;
        }
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
        let error = gl.get_error();
        if error != glow::NO_ERROR {
            return Err(format!("GL error 0x{error:x} uploading the scene"));
        }
    }
    Ok(job.uploaded >= total)
}

/// One `RGBA8` renderbuffer and the framebuffer it is attached to.
struct Target {
    framebuffer: glow::Framebuffer,
    renderbuffer: glow::Renderbuffer,
}

impl Target {
    fn new(gl: &glow::Context, [width, height]: [i32; 2]) -> Result<Self, String> {
        // SAFETY: on the caller's context; both objects are returned or deleted.
        unsafe {
            let renderbuffer = gl.create_renderbuffer()?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::RGBA8, width, height);
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            let framebuffer = match gl.create_framebuffer() {
                Ok(f) => f,
                Err(e) => {
                    gl.delete_renderbuffer(renderbuffer);
                    return Err(e);
                }
            };
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(renderbuffer),
            );
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.delete_framebuffer(framebuffer);
                gl.delete_renderbuffer(renderbuffer);
                return Err(format!("scene target incomplete: 0x{status:x}"));
            }
            Ok(Self {
                framebuffer,
                renderbuffer,
            })
        }
    }

    fn destroy(self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_renderbuffer(self.renderbuffer);
        }
    }
}
