//! The live picture drawn by the graphics card: the engine's draw data and
//! GLSL ES 3.00 shaders, through `native_mesh`, in an `egui_glow` paint
//! callback. The CPU renderer still draws every picture written out, and the
//! live one when this cannot start (the driver refused the shaders or an
//! offscreen buffer); `PDFCER_3D_CPU=1` forces it, for comparison.
//!
//! A camera move costs only new uniforms; the vertices are rebuilt when the
//! shown or highlighted parts or the lighting change, and the pictures when
//! the model does. The shadow maps are redrawn when the engine's shadows for
//! the frame change or the vertices do; ambient occlusion runs every picture.
//! A paint with nothing new copies the last picture again.
//!
//! The card draws everything the engine's shaders do: the model's own
//! lights, lamp shadows, every directional caster, see-through shadows,
//! material finishes, the emissive repeat, and lines and points of any
//! width. A frame whose points are wider than the card's widest is
//! declined, and the viewer draws it on the processor and says so under the
//! picture. The passes themselves are `gpu_paint`'s.

use std::sync::{Arc, Mutex, PoisonError};

use eframe::egui_glow;
use native_mesh::{Draw, Mesh, Value, Wrap};
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    AssembledModel, Camera, DrawData, FrameAmbientOcclusion, FrameExtraShadows, FrameLampShadows,
    FrameLights, FrameShadow, PreparedModel, RenderError, RenderMode, ShininessScale, TextureWrap,
};

use super::effects::Effects;
use super::gpu_uniforms as u;
use super::{ModelView, Rendered};
use crate::text::model3dlook::ProcessorReason;

/// GL objects of viewers closed since the last paint; the next paint of any
/// viewer deletes them, since only a paint holds the context.
pub(super) static RETIRED: Mutex<Vec<Mesh>> = Mutex::new(Vec::new());

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
    /// What the uploaded draws hold that the frame's passes depend on.
    facts: Facts,
    /// Why the last frame was left to the processor.
    declined: Option<ProcessorReason>,
}

/// What the uploaded draws hold that the frame's passes depend on.
#[derive(Debug, Clone, Default)]
struct Facts {
    /// A translucent draw, which casts under see-through shadows.
    translucent: bool,
    /// The emissive repeat's share; 0 for none.
    emissive: f32,
    /// The points' size in pixels, when there are points.
    point_size: Option<f32>,
    /// The line and point programs' uniforms past the camera's.
    lines: u::Uniforms,
}

/// What a set of draws depends on besides the model.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Built {
    parts: u32,
    accent: egui::Color32,
    lighting: pdfcer_3d::LightingScheme,
    mode: RenderMode,
    shininess: ShininessScale,
}

/// What the paint callback reads, written by the viewer each frame.
#[derive(Default)]
pub(super) struct Shared {
    pub(super) mesh: Option<Mesh>,
    pub(super) failed: Option<String>,
    /// Vertices, and pictures when the model changed, for the next paint.
    pub(super) upload: Option<(Vec<f32>, Option<Vec<Owned>>)>,
    /// The line segments' and points' vertices, for the next paint.
    pub(super) overlay_upload: Option<[Vec<f32>; 2]>,
    /// Instanced segments drawn.
    pub(super) segments: i32,
    /// Points drawn.
    pub(super) points: i32,
    pub(super) draws: Vec<Draw>,
    pub(super) uniforms: u::Uniforms,
    /// The line and point programs' uniforms but `u_viewport`.
    pub(super) line_uniforms: u::Uniforms,
    pub(super) background: [f32; 4],
    /// `u_lights`, `pdfcer_3d::LIGHT_TABLE_WIDTH` texels a row.
    pub(super) light_table: Vec<f32>,
    /// The key light's shadow, when one is cast.
    pub(super) shadow: Option<FrameShadow>,
    /// The further directional casters.
    pub(super) extra: Option<FrameExtraShadows>,
    /// The lamps' cube shadows.
    pub(super) lamps: Option<FrameLampShadows>,
    /// The alpha range a translucent caster's solid depth keeps, when
    /// translucent draws cast.
    pub(super) keep: Option<[f32; 2]>,
    /// The alpha range drawn into the see-through layers, when any.
    pub(super) veil: Option<[f32; 2]>,
    /// The directional maps changed since last drawn.
    pub(super) directional_redraw: bool,
    /// The lamp cubes changed since last drawn.
    pub(super) lamp_redraw: bool,
    /// The faces are drawn after a depth prepass.
    pub(super) prepass: bool,
    /// The frame's ambient occlusion, when drawn.
    pub(super) occlusion: Option<FrameAmbientOcclusion>,
    /// The emissive repeat's uniforms and share.
    pub(super) repeat: Option<(u::Uniforms, f32)>,
    /// The widest point the card draws, pixels; 0 before the first paint.
    pub(super) widest_point: f32,
    /// The trace line's head, when the picture is new.
    pub(super) trace: Option<String>,
    /// The mesh holds the picture of everything above: set by a draw,
    /// cleared by the viewer's every write. A paint while it is set copies
    /// that picture again, so a repaint of the window around an unmoved
    /// model costs a blit rather than every pass.
    pub(super) drawn: bool,
}

/// A texture's pixels, held until the paint uploads them.
pub(super) struct Owned {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
    pub(super) wrap: [Wrap; 2],
}

/// The shadow inputs of one frame, before they reach [`Shared`].
struct Casters {
    shadow: Option<FrameShadow>,
    extra: Option<FrameExtraShadows>,
    lamps: Option<FrameLampShadows>,
    keep: Option<[f32; 2]>,
    veil: Option<[f32; 2]>,
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            shared: Arc::default(),
            prepared: None,
            built: None,
            forced_off: std::env::var_os("PDFCER_3D_CPU").is_some_and(|v| v == "1"),
            told: false,
            facts: Facts::default(),
            declined: None,
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

    /// Whether the card draws, or will try to: not forced off, not failed.
    pub(super) fn drawing(&self) -> bool {
        !self.forced_off && lock(&self.shared).failed.is_none()
    }

    /// Whether the card failed, so the processor draws every frame.
    pub(super) fn failed(&self) -> bool {
        !self.forced_off && lock(&self.shared).failed.is_some()
    }

    /// Why the last frame was left to the processor.
    pub(super) fn declined(&self) -> Option<ProcessorReason> {
        self.declined
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
            super::gpu_paint::draw(&shared, &info, painter);
        });
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        });
    }

    /// Why the card cannot draw the uploaded draws: points wider than its
    /// widest, once a paint has measured it.
    fn decline(&self) -> Option<ProcessorReason> {
        let widest = lock(&self.shared).widest_point;
        let size = self.facts.point_size?;
        (widest > 0.0 && size > widest).then_some(ProcessorReason::WidePoints)
    }
}

impl ModelView {
    /// Set the GPU picture up for `wanted`: new draws when the parts,
    /// lighting, render mode or highlights changed, new uniforms always,
    /// and the trace line's head. `false` declines the frame: it shows
    /// what the card does not draw, and the processor must draw it.
    pub(super) fn gpu_frame(&mut self, wanted: Rendered) -> Result<bool, RenderError> {
        let options = wanted.look.options();
        let built = Built {
            parts: wanted.parts,
            accent: wanted.accent,
            lighting: wanted.look.lighting,
            mode: options.render_mode,
            shininess: options.shininess,
        };
        if self.gpu.built != Some(built) {
            self.gpu_scene(&options, wanted.accent);
            self.gpu.built = Some(built);
        }
        self.gpu.declined = self.gpu.decline();
        if self.gpu.declined.is_some() {
            return Ok(false);
        }
        let [width, height] = wanted.size;
        let aspect = f64::from(width) / f64::from(height.max(1));
        let camera = self.view_camera(aspect)?;
        let lights = &self.model.drawn.lights;
        let effects = Effects::new(&camera, &options, &self.bounds, lights, wanted.size)?;
        let frame_lights = FrameLights::for_model(&camera, &options, lights)?;
        let casters = self.casters(&camera, &options, effects.shadow)?;
        let view = u::camera_uniforms(&camera, &frame_lights, aspect, &self.bounds)?;
        let mut uniforms = view.clone();
        uniforms.extend(u::light_uniforms(&frame_lights));
        uniforms.extend(u::shadow_uniforms(casters.shadow.as_ref()));
        uniforms.extend(u::extra_uniforms(casters.extra.as_ref()));
        uniforms.extend(u::lamp_uniforms(casters.lamps.as_ref()));
        let veiled = casters.veil.is_some();
        uniforms.extend(u::veil_uniforms(
            veiled && casters.shadow.is_some(),
            veiled && casters.lamps.is_some(),
        ));
        let emissive = self.gpu.facts.emissive;
        let repeat = if emissive > 0.0 {
            Some((u::repeat_uniforms(&camera, &options)?, emissive))
        } else {
            None
        };
        let mut line_uniforms = view;
        line_uniforms.extend(self.gpu.facts.lines.iter().cloned());
        let head = self.trace_head(&wanted, Ok(&camera), &options) + &casters.trace();
        let mut shared = lock(&self.gpu.shared);
        shared.uniforms = uniforms;
        shared.line_uniforms = line_uniforms;
        shared.light_table = frame_lights.light_table();
        shared.set_casters(casters);
        shared.occlusion = effects.occlusion;
        shared.repeat = repeat;
        shared.prepass = wanted.look.extras.skip_hidden;
        shared.background = wanted.background.to_normalized_gamma_f32();
        shared.trace = crate::diag::enabled().then_some(head);
        shared.drawn = false;
        Ok(true)
    }

    /// The frame's shadow casters past the key light's `shadow`: further
    /// directional lights only beside it, lamps on their own, and the
    /// see-through ranges when a translucent draw casts.
    fn casters(
        &self,
        camera: &Camera,
        options: &ModelRenderOptions,
        shadow: Option<FrameShadow>,
    ) -> Result<Casters, RenderError> {
        let lights = &self.model.drawn.lights;
        let extra = match shadow {
            Some(_) => FrameExtraShadows::for_model(camera, options, &self.bounds, lights)?,
            None => None,
        };
        let lamps = FrameLampShadows::for_model(camera, options, &self.bounds, lights)?;
        let casting = (shadow.is_some() || lamps.is_some()) && self.gpu.facts.translucent;
        Ok(Casters {
            shadow,
            extra,
            lamps,
            keep: options.shadow_alpha.solid_range().filter(|_| casting),
            veil: options.shadow_alpha.veil_range().filter(|_| casting),
        })
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
        self.gpu.facts = Facts {
            translucent: data.draws.iter().any(|d| d.translucent),
            emissive: data.emissive,
            point_size: (!data.points.is_empty()).then_some(data.point_size),
            lines: u::line_uniforms(&data),
        };
        let draws = self.gpu_draws(&data);
        let mut shared = lock(&self.gpu.shared);
        shared.draws = draws;
        shared.segments = (data.lines.len() / (2 * pdfcer_3d::LINE_VERTEX_FLOATS)) as i32;
        shared.points = (data.points.len() / pdfcer_3d::LINE_VERTEX_FLOATS) as i32;
        shared.overlay_upload = Some([data.lines, data.points]);
        shared.upload = Some((data.vertices, pictures));
    }

    /// The mesh's draws for `data`, each with its own uniforms.
    fn gpu_draws(&self, data: &DrawData) -> Vec<Draw> {
        let textures = &self.model.drawn.textures;
        data.draws
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
                    uniforms: [
                        ("u_colour", Value::Vec4(d.colour)),
                        ("u_textured", Value::Int(i32::from(picture.is_some()))),
                        ("u_function", Value::Int(function)),
                        ("u_blend_colour", Value::Vec4(blend)),
                    ]
                    .into_iter()
                    .chain(u::finish_uniforms(d.finish.as_ref()))
                    .collect(),
                }
            })
            .collect()
    }
}

impl Casters {
    /// The render trace's fields for the card's casters.
    fn trace(&self) -> String {
        let extra = self.extra.map_or(0, |e| e.count);
        let lamps = self
            .lamps
            .map_or(0, |l| l.far.iter().filter(|&&f| f > 0.0).count());
        let veil = u8::from(self.veil.is_some());
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " gpu-extra={extra} gpu-lamps={lamps} gpu-veil={veil}"
        )
    }
}

impl Shared {
    /// Take the frame's casters, marking the maps whose inputs changed.
    fn set_casters(&mut self, c: Casters) {
        let directional = (self.shadow, self.extra, self.keep, self.veil);
        self.directional_redraw |= directional != (c.shadow, c.extra, c.keep, c.veil);
        let lamp = (self.lamps, self.keep, self.veil);
        self.lamp_redraw |= lamp != (c.lamps, c.keep, c.veil);
        self.shadow = c.shadow;
        self.extra = c.extra;
        self.lamps = c.lamps;
        self.keep = c.keep;
        self.veil = c.veil;
    }
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

pub(super) fn lock(shared: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}
