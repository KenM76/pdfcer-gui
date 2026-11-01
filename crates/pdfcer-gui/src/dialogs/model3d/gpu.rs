//! The live picture drawn by the graphics card: the engine's draw data and
//! GLSL ES 3.00 shaders, through `native_mesh`, in an `egui_glow` paint
//! callback. The CPU renderer still draws every picture written out, and the
//! live one when this cannot start (the driver refused the shaders or an
//! offscreen buffer); `PDFCER_3D_CPU=1` forces it, for comparison.
//!
//! A camera move costs only new uniforms; the vertices are rebuilt when the
//! shown or highlighted parts or the lighting change, and the pictures when
//! the model does.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use eframe::egui_glow;
use native_mesh::{Draw, Mesh, Picture, Target, Value, Wrap};
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    AssembledModel, Bounds, Camera, FrameLights, GLSL_ES_300_FRAGMENT, GLSL_ES_300_VERTEX,
    MAX_LIGHTS, PreparedModel, RenderError, TextureWrap,
};

use super::{ModelView, Rendered};

/// Vertex attribute widths, matching `pdfcer_3d::VERTEX_FLOATS`: position,
/// normal, uv, flat.
const ATTRIBUTES: [i32; 4] = [3, 3, 2, 1];
const _: () = assert!(3 + 3 + 2 + 1 == pdfcer_3d::VERTEX_FLOATS);

/// GL objects of viewers closed since the last paint; the next paint of any
/// viewer deletes them, since only a paint holds the context.
static RETIRED: Mutex<Vec<Mesh>> = Mutex::new(Vec::new());

/// The viewer's side of the GPU picture.
pub(super) struct Gpu {
    shared: Arc<Mutex<Shared>>,
    /// Kept for the model's life; dropped when a mesh fit replaces it.
    prepared: Option<PreparedModel>,
    /// What the uploaded draws were built for.
    built: Option<Built>,
    /// `PDFCER_3D_CPU` asked for the CPU renderer.
    forced_off: bool,
    /// The failure was traced.
    told: bool,
}

/// What a set of draws depends on besides the model.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Built {
    parts: u32,
    accent: egui::Color32,
    lighting: pdfcer_3d::LightingScheme,
}

/// What the paint callback reads, written by the viewer each frame.
#[derive(Default)]
struct Shared {
    mesh: Option<Mesh>,
    failed: Option<String>,
    /// Vertices, and pictures when the model changed, for the next paint.
    upload: Option<(Vec<f32>, Option<Vec<Owned>>)>,
    draws: Vec<Draw>,
    uniforms: Vec<(&'static str, Value)>,
    background: [f32; 4],
    /// The trace line's head, when the picture is new.
    trace: Option<String>,
}

/// A texture's pixels, held until the paint uploads them.
struct Owned {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    wrap: [Wrap; 2],
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            shared: Arc::default(),
            prepared: None,
            built: None,
            forced_off: std::env::var_os("PDFCER_3D_CPU").is_some_and(|v| v == "1"),
            told: false,
        }
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        let mesh = lock(&self.shared).mesh.take();
        if let Some(mesh) = mesh {
            RETIRED
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(mesh);
        }
    }
}

impl Gpu {
    /// Whether the GPU draws the live picture.
    pub(super) fn live(&mut self) -> bool {
        if self.forced_off {
            return false;
        }
        let failed = lock(&self.shared).failed.clone();
        if let Some(error) = failed {
            if !self.told {
                self.told = true;
                crate::diag::trace(|| format!("model-view-gpu failed error={error:?}")); // ui-text-exempt: diagnostic trace, never displayed
            }
            return false;
        }
        true
    }

    /// The model was replaced: rebuild everything from it.
    pub(super) fn forget_model(&mut self) {
        self.prepared = None;
        self.built = None;
    }

    /// Queue the picture for `rect`, drawn when egui paints.
    pub(super) fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        let shared = Arc::clone(&self.shared);
        let callback = egui_glow::CallbackFn::new(move |info, painter| {
            draw(&shared, &info, painter);
        });
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        });
    }
}

impl ModelView {
    /// Set the GPU picture up for `wanted`: new draws when the parts or
    /// lighting changed, new uniforms always, and the trace line's head.
    pub(super) fn gpu_frame(&mut self, wanted: Rendered) -> Result<(), RenderError> {
        let options = ModelRenderOptions {
            lighting: wanted.lighting,
            light_frame: wanted.light_frame,
            ..ModelRenderOptions::default()
        };
        let built = Built {
            parts: wanted.parts,
            accent: wanted.accent,
            lighting: wanted.lighting,
        };
        if self.gpu.built != Some(built) {
            self.gpu_scene(&options, wanted.accent);
            self.gpu.built = Some(built);
        }
        let [width, height] = wanted.size;
        let aspect = f64::from(width) / f64::from(height.max(1));
        let camera = self.view_camera(aspect)?;
        let uniforms = frame_uniforms(&camera, &options, aspect, &self.bounds)?;
        let head = self.trace_head(&wanted, Ok(&camera), wanted.lighting);
        let mut shared = lock(&self.gpu.shared);
        shared.uniforms = uniforms;
        shared.background = wanted.background.to_normalized_gamma_f32();
        shared.trace = crate::diag::enabled().then_some(head);
        Ok(())
    }

    /// Rebuild the draws from the model painted for the parts' state.
    fn gpu_scene(&mut self, options: &ModelRenderOptions, accent: egui::Color32) {
        let pictures = self
            .gpu
            .prepared
            .is_none()
            .then(|| owned_pictures(&self.model.drawn));
        let prepared = self
            .gpu
            .prepared
            .get_or_insert_with(|| PreparedModel::new(&self.model.drawn));
        let data = self
            .parts
            .painted(&mut self.model.drawn, Some(accent.to_array()), |model| {
                prepared.draw_data(model, options)
            });
        let textures = &self.model.drawn.textures;
        let draws = data
            .draws
            .iter()
            .map(|d| {
                let picture = d.texture.filter(|&t| t < textures.len());
                let (function, blend) = picture
                    .and_then(|t| textures.get(t))
                    .map_or((0, [0.0; 4]), |t| t.function.shader_code());
                Draw {
                    first: d.first as i32,
                    count: d.count as i32,
                    picture,
                    translucent: d.translucent,
                    uniforms: vec![
                        ("u_colour", Value::Vec4(d.colour)),
                        ("u_textured", Value::Int(i32::from(picture.is_some()))),
                        ("u_function", Value::Int(function)),
                        ("u_blend_colour", Value::Vec4(blend)),
                    ],
                }
            })
            .collect();
        let mut shared = lock(&self.gpu.shared);
        shared.draws = draws;
        shared.upload = Some((data.vertices, pictures));
    }
}

/// The per-frame uniforms: the camera's matrices and the lights it sees.
fn frame_uniforms(
    camera: &Camera,
    options: &ModelRenderOptions,
    aspect: f64,
    bounds: &Bounds,
) -> Result<Vec<(&'static str, Value)>, RenderError> {
    let lights = FrameLights::new(camera, options)?;
    let count = lights.count.min(MAX_LIGHTS);
    Ok(vec![
        ("u_view", Value::Mat4(camera.view_matrix()?)),
        (
            "u_projection",
            Value::Mat4(camera.projection_matrix(aspect, bounds)?),
        ),
        (
            "u_depth_range",
            Value::Vec2(camera.depth_range(aspect, bounds)?),
        ),
        ("u_mode", Value::Int(lights.mode.code())),
        ("u_ambient", Value::Vec3(lights.ambient)),
        ("u_count", Value::Int(count as i32)),
        ("u_light_dir", Value::Vec4s(lights.directions.concat())),
        ("u_light_colour", Value::Vec3s(lights.colours.concat())),
        ("u_eye", Value::Vec3(lights.eye)),
        ("u_forward", Value::Vec3(lights.forward)),
        ("u_orthographic", Value::Int(i32::from(lights.orthographic))),
        ("u_texture", Value::Int(0)),
    ])
}

/// Every texture of `model`, copied for the upload.
fn owned_pictures(model: &AssembledModel) -> Vec<Owned> {
    let wrap = |w: TextureWrap| match w {
        TextureWrap::Clamp => Wrap::Clamp,
        TextureWrap::MirroredRepeat => Wrap::Mirrored,
        _ => Wrap::Repeat,
    };
    model
        .textures
        .iter()
        .map(|t| Owned {
            width: t.width,
            height: t.height,
            rgba: t.rgba.clone(),
            wrap: t.wrap.map(wrap),
        })
        .collect()
}

/// The paint callback: create or update the GL objects and draw.
fn draw(shared: &Mutex<Shared>, info: &egui::PaintCallbackInfo, painter: &egui_glow::Painter) {
    let gl = painter.gl();
    for mesh in RETIRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .drain(..)
    {
        mesh.destroy(gl);
    }
    let mut guard = lock(shared);
    let s = &mut *guard;
    if s.failed.is_some() {
        return;
    }
    if s.mesh.is_none() {
        match Mesh::new(gl, GLSL_ES_300_VERTEX, GLSL_ES_300_FRAGMENT, &ATTRIBUTES) {
            Ok(mesh) => s.mesh = Some(mesh),
            Err(error) => {
                s.failed = Some(error);
                return;
            }
        }
    }
    let Some(mesh) = s.mesh.as_mut() else { return };
    if let Some((vertices, pictures)) = s.upload.take() {
        mesh.set_vertices(gl, &vertices);
        if let Some(pictures) = pictures {
            let borrowed: Vec<Picture> = pictures
                .iter()
                .map(|p| Picture {
                    width: p.width,
                    height: p.height,
                    rgba: &p.rgba,
                    wrap: p.wrap,
                })
                .collect();
            if let Err(error) = mesh.set_pictures(gl, &borrowed) {
                s.failed = Some(error);
                return;
            }
        }
    }
    let started = Instant::now();
    let vp = info.viewport_in_pixels();
    let clip = info.clip_rect_in_pixels();
    let target = Target {
        viewport: [vp.left_px, vp.from_bottom_px, vp.width_px, vp.height_px],
        clip: [
            clip.left_px,
            clip.from_bottom_px,
            clip.width_px,
            clip.height_px,
        ],
        destination: painter.intermediate_fbo(),
        background: s.background,
        read_back: s.trace.is_some(),
    };
    match mesh.draw(gl, &s.uniforms, &s.draws, &target) {
        Ok(Some(rgba)) => {
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            if let Some(head) = s.trace.take() {
                let fields = super::picture::pixel_fields(&rgba, to_bytes(s.background));
                crate::diag::trace(|| format!("{head} gpu=1 ms={ms:.1}{fields}")); // ui-text-exempt: diagnostic trace, never displayed
            }
        }
        Ok(None) => {}
        Err(error) => s.failed = Some(error),
    }
}

/// `colour` 0–1 as the bytes a read-back holds.
fn to_bytes(colour: [f32; 4]) -> [u8; 4] {
    colour.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn lock(shared: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}
