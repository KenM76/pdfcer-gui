//! A fisheye frame ([`crate::Lens`]): each face drawn single-sample at its
//! own size, its colour and depth copied out, then resampled into the
//! picture's multisampled buffer before the next face is drawn, so only one
//! face's buffers are live at a time. Face buffers are kept by size between
//! frames and dropped once a frame no longer uses their size.

use glow::HasContext as _;

use crate::buffers::{Buffers, ScreenTextures};
use crate::program::Program;
use crate::{Draw, Lens, LensFace, Mesh, Passes, Target, Value};

/// The lens program and the face buffers, by size.
#[derive(Default)]
pub(crate) struct LensTargets {
    program: Option<(Program, glow::VertexArray)>,
    /// A face's buffers and the textures its frame is copied into, `None`
    /// only while lent to a screen effect.
    faces: Vec<(Buffers, Option<ScreenTextures>)>,
}

impl LensTargets {
    /// The face buffers of `size`, made when none are kept.
    fn fit(&mut self, gl: &glow::Context, size: [i32; 2]) -> Result<usize, String> {
        if let Some(i) = self.faces.iter().position(|(b, _)| b.size == size) {
            return Ok(i);
        }
        let buffers = Buffers::new(gl, size, 0)?;
        let textures = match ScreenTextures::new(gl, size) {
            Ok(t) => t,
            Err(e) => {
                buffers.delete(gl);
                return Err(e);
            }
        };
        self.faces.push((buffers, Some(textures)));
        Ok(self.faces.len() - 1)
    }

    /// Drop the face buffers no face of `lens` is drawn at.
    fn keep_only(&mut self, gl: &glow::Context, lens: &Lens<'_>) {
        let (kept, dropped): (Vec<_>, Vec<_>) = std::mem::take(&mut self.faces)
            .into_iter()
            .partition(|(b, _)| lens.faces.iter().any(|f| f.size == b.size));
        for (b, t) in dropped {
            b.delete(gl);
            t.into_iter().for_each(|t| t.delete(gl));
        }
        self.faces = kept;
    }

    /// Delete every object.
    pub(crate) fn delete(self, gl: &glow::Context) {
        for (b, t) in self.faces {
            b.delete(gl);
            t.into_iter().for_each(|t| t.delete(gl));
        }
        if let Some((program, vao)) = self.program {
            // SAFETY: the vertex array is this value's own.
            unsafe { gl.delete_vertex_array(vao) };
            program.delete(gl);
        }
    }
}

impl Mesh {
    /// Link the program a [`Lens`] resamples its faces with: `vertex`
    /// reads no attributes and places three vertices covering the picture.
    ///
    /// # Errors
    ///
    /// The driver's compile or link log, or the object it could not create.
    pub fn set_lens_program(
        &mut self,
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
    ) -> Result<(), String> {
        let program = Program::new(gl, vertex, fragment)?;
        // SAFETY: on the caller's current context; kept in this mesh.
        let vao = match unsafe { gl.create_vertex_array() } {
            Ok(v) => v,
            Err(e) => {
                program.delete(gl);
                return Err(e);
            }
        };
        if let Some((old, old_vao)) = self.lens.program.replace((program, vao)) {
            // SAFETY: the vertex array was this mesh's own.
            unsafe { gl.delete_vertex_array(old_vao) };
            old.delete(gl);
        }
        Ok(())
    }

    /// [`Mesh::draw`] for a fisheye frame; the inputs are filled.
    pub(crate) fn draw_lens(
        &mut self,
        gl: &glow::Context,
        uniforms: &[(&'static str, Value)],
        draws: &[Draw],
        passes: &Passes<'_>,
        lens: &Lens<'_>,
        target: &Target,
    ) -> Result<Option<Vec<u8>>, String> {
        if self.lens.program.is_none() {
            return Err("fisheye frame asked for with no lens program".to_owned());
        }
        let Some(picture) = self.buffers.as_ref().map(|b| (b.multisampled, b.size)) else {
            return Ok(None);
        };
        self.lens.keep_only(gl, lens);
        let mut targets = std::mem::take(&mut self.lens);
        let mut drawn = Ok(());
        self.begin(gl, picture.0, picture.1, target.background);
        for face in lens.faces {
            drawn = self.draw_face(gl, &mut targets, (uniforms, draws, passes), face, target);
            if drawn.is_err() {
                break;
            }
            resample(gl, &mut targets, lens, face, picture);
        }
        self.lens = targets;
        drawn?;
        // SAFETY: the picture's framebuffer is this mesh's own.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(picture.0));
            gl.viewport(0, 0, picture.1[0], picture.1[1]);
        }
        crate::overlay::draw_all(gl, &mut self.overlays, passes.overlays);
        let Some(b) = &self.buffers else {
            return Ok(None);
        };
        b.resolve(gl);
        Ok(crate::buffers::present(gl, b, target))
    }

    /// Draw `face` into its buffers and copy its colour and depth out.
    fn draw_face(
        &mut self,
        gl: &glow::Context,
        targets: &mut LensTargets,
        (uniforms, draws, passes): (&[(&'static str, Value)], &[Draw], &Passes<'_>),
        face: &LensFace<'_>,
        target: &Target,
    ) -> Result<(), String> {
        let [w, h] = face.size;
        if w <= 0 || h <= 0 {
            return Ok(());
        }
        let i = targets.fit(gl, face.size)?;
        let Some((buffers, textures)) = targets.faces.get_mut(i) else {
            return Ok(());
        };
        self.begin(gl, buffers.multisampled, face.size, target.background);
        for (name, value) in uniforms.iter().chain(face.uniforms) {
            self.main.set(gl, name, value);
        }
        self.bind_inputs(gl, passes);
        self.draw_runs(gl, draws, None);
        if let Some(effect) = &face.screen {
            self.unbind_inputs(gl, passes);
            // The effect copies the frame into the screen textures; the
            // face's own are of its size, so lend them.
            std::mem::swap(&mut self.screen_textures, textures);
            let ran = self.screen(gl, effect, buffers);
            std::mem::swap(&mut self.screen_textures, textures);
            ran?;
        }
        if let Some(repeat) = &face.repeat {
            self.repeat(gl, buffers.multisampled, face.size, passes, repeat, draws);
        }
        self.unbind_inputs(gl, passes);
        if let Some(t) = textures {
            t.take_frame(gl, buffers);
        }
        Ok(())
    }
}

/// Resample `face`'s copied frame into the picture's buffer, `picture` its
/// framebuffer and size, writing colour and depth at the pixels it owns.
fn resample(
    gl: &glow::Context,
    targets: &mut LensTargets,
    lens: &Lens<'_>,
    face: &LensFace<'_>,
    (framebuffer, [w, h]): (glow::Framebuffer, [i32; 2]),
) {
    let Some(textures) = targets
        .faces
        .iter()
        .find(|(b, _)| b.size == face.size)
        .and_then(|(_, t)| t.as_ref())
        .map(|t| (t.colour, t.depth))
    else {
        return;
    };
    let Some((program, vao)) = targets.program.as_mut() else {
        return;
    };
    // SAFETY: the framebuffer, textures, program and vertex array are the
    // mesh's own; units 0 and 1 are rebound to none before returning.
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.viewport(0, 0, w, h);
        gl.disable(glow::SCISSOR_TEST);
        gl.disable(glow::BLEND);
        gl.disable(glow::CULL_FACE);
        // GL writes no depth while the test is off; the picture's depth is
        // what hides the overlays behind the faces.
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::ALWAYS);
        gl.depth_mask(true);
        gl.color_mask(true, true, true, true);
        for (unit, t) in [textures.0, textures.1].into_iter().enumerate() {
            gl.active_texture(glow::TEXTURE0 + unit as u32);
            gl.bind_texture(glow::TEXTURE_2D, Some(t));
        }
        program.use_with(gl, lens.uniforms);
        for (name, value) in face.resample {
            program.set(gl, name, value);
        }
        program.set(gl, lens.colour, &Value::Int(0));
        program.set(gl, lens.depth, &Value::Int(1));
        gl.bind_vertex_array(Some(*vao));
        gl.draw_arrays(glow::TRIANGLES, 0, 3);
        gl.bind_vertex_array(None);
        for unit in (0..2).rev() {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
        gl.depth_func(glow::LESS);
    }
}
