//! A caller's one-triangle program drawn over an `RGBA32I` scene texture into
//! `RGBA8` framebuffers, tile by tile, and the pixels read back.
//!
//! Contract: a [`SceneDrawer`] is made and used on the context the caller
//! paints with, and [`SceneDrawer::destroy`]ed on it. The program reads the
//! scene through the sampler `u_scene` on unit 0 and is told each tile's
//! first pixel through the `ivec2` uniform `u_origin`, so framebuffer pixel
//! `(x, y)` is output pixel `u_origin + (x, y)`. Row `y` of each read tile is
//! taken as output row `u_origin.y + y`: the program, not this module, decides
//! which way up the picture is. Each tile is drawn in bands of
//! [`BAND_ROWS`] rows, so one submission stays short on a slow or software
//! GPU.
//!
//! State left behind: no framebuffer, program, vertex array, renderbuffer or
//! texture bound, scissor and blending off. `egui_glow` re-prepares its own
//! state each frame.

use glow::HasContext as _;

use crate::Value;
use crate::program::Program;

/// Rows a band submits at once.
pub const BAND_ROWS: u32 = 256;

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
    /// bytes, row 0 first.
    ///
    /// # Errors
    ///
    /// A texture the shape of which disagrees with its values, an incomplete
    /// framebuffer, or a GL error raised while drawing.
    pub fn draw(
        &mut self,
        gl: &glow::Context,
        scene: SceneTexture<'_>,
        [width, height]: [u32; 2],
    ) -> Result<Vec<u8>, String> {
        let want = scene.width as usize * scene.height as usize * scene.layers as usize * 4;
        if want == 0 || scene.texels.len() != want {
            return Err(format!(
                "a scene of {} values is not {want}",
                scene.texels.len()
            ));
        }
        if width == 0 || height == 0 {
            return Err("an empty picture".to_owned());
        }
        // SAFETY: a plain error-queue drain on the caller's context.
        unsafe { while gl.get_error() != glow::NO_ERROR {} }
        let texture = upload(gl, scene)?;
        let tile = tile_edge(gl);
        let result = self.draw_tiles(gl, texture, tile, [width, height]);
        // SAFETY: the texture is this call's own.
        unsafe { gl.delete_texture(texture) };
        result
    }

    fn draw_tiles(
        &mut self,
        gl: &glow::Context,
        texture: glow::Texture,
        tile: i32,
        [width, height]: [u32; 2],
    ) -> Result<Vec<u8>, String> {
        let (w, h) = (width as i32, height as i32);
        let target = Target::new(gl, [tile.min(w), tile.min(h)])?;
        let mut out = vec![0_u8; width as usize * height as usize * 4];
        let mut part = Vec::new();
        self.program.use_with(gl, &[("u_scene", Value::Int(0))]);
        // SAFETY: every object named is this value's or this call's own, on
        // the caller's context; the state set here is reset below.
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(target.framebuffer));
            gl.bind_vertex_array(Some(self.vao));
            gl.disable(glow::BLEND);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::SCISSOR_TEST);
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
        }
        for y0 in (0..h).step_by(tile as usize) {
            for x0 in (0..w).step_by(tile as usize) {
                let (tw, th) = (tile.min(w - x0), tile.min(h - y0));
                self.program.set(gl, "u_origin", &Value::IVec2([x0, y0]));
                part.resize(tw as usize * th as usize * 4, 0);
                // SAFETY: as above; `part` holds exactly `tw × th` RGBA8 pixels.
                unsafe {
                    gl.viewport(0, 0, tw, th);
                    for band in (0..th).step_by(BAND_ROWS as usize) {
                        gl.scissor(0, band, tw, (BAND_ROWS as i32).min(th - band));
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);
                        gl.flush();
                    }
                    gl.read_pixels(
                        0,
                        0,
                        tw,
                        th,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        glow::PixelPackData::Slice(Some(&mut part)),
                    );
                }
                place(&mut out, width, &part, [x0, y0], [tw, th]);
            }
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
        target.destroy(gl);
        if error == glow::NO_ERROR {
            Ok(out)
        } else {
            Err(format!("GL error 0x{error:x} while drawing"))
        }
    }

    /// Delete every GL object.
    pub fn destroy(self, gl: &glow::Context) {
        self.program.delete(gl);
        // SAFETY: the array is this value's own.
        unsafe { gl.delete_vertex_array(self.vao) };
    }
}

/// Copy a `tw` × `th` tile read at `(x0, y0)` into `out`, `width` pixels a row.
fn place(out: &mut [u8], width: u32, part: &[u8], [x0, y0]: [i32; 2], [tw, th]: [i32; 2]) {
    let row = tw as usize * 4;
    for y in 0..th as usize {
        let at = ((y0 as usize + y) * width as usize + x0 as usize) * 4;
        out[at..at + row].copy_from_slice(&part[y * row..(y + 1) * row]);
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
fn upload(gl: &glow::Context, scene: SceneTexture<'_>) -> Result<glow::Texture, String> {
    let bytes: Vec<u8> = scene.texels.iter().flat_map(|v| v.to_ne_bytes()).collect();
    // SAFETY: on the caller's context; `bytes` is exactly the array.
    unsafe {
        let texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
        crate::compute::nearest(gl, glow::TEXTURE_2D_ARRAY);
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
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
            glow::PixelUnpackData::Slice(Some(&bytes)),
        );
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
        let error = gl.get_error();
        if error != glow::NO_ERROR {
            gl.delete_texture(texture);
            return Err(format!("GL error 0x{error:x} uploading the scene"));
        }
        Ok(texture)
    }
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
