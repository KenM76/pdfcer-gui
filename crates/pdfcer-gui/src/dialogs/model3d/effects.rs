//! What the live picture is drawn with: the lighting, and the shadows and
//! ambient occlusion the engine adds although the PDF standard defines
//! neither. Their switches sit under the lighting drop-down, and which light
//! casts, or why nothing does, is said there, never drawn into the picture.

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    Bounds, Camera, FrameAmbientOcclusion, FrameShadow, LightFrame, LightSense, LightingScheme,
    RenderError, ShadowLight,
};

use super::ModelView;
use crate::text::model3d as m;

/// The shadows switch.
pub const REGION_SHADOWS: &str = "model3d.shadows"; // ui-text-exempt: trace region name, never displayed
/// The ambient occlusion switch.
pub const REGION_OCCLUSION: &str = "model3d.occlusion"; // ui-text-exempt: trace region name, never displayed
/// The drop-down for which way a scheme's light directions point.
pub const REGION_LIGHT_SENSE: &str = "model3d.light_sense"; // ui-text-exempt: trace region name, never displayed
/// The drop-down for which light casts shadows.
pub const REGION_SHADOW_LIGHT: &str = "model3d.shadow_light"; // ui-text-exempt: trace region name, never displayed

/// Everything the picture's look depends on besides the model and camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Look {
    pub(super) lighting: LightingScheme,
    pub(super) light_frame: LightFrame,
    pub(super) shadows: bool,
    pub(super) occlusion: bool,
    pub(super) light_sense: LightSense,
    pub(super) shadow_light: ShadowLight,
}

impl Look {
    /// The renderer's options for this look; size, background and threads
    /// are the caller's.
    pub(super) fn options(self) -> ModelRenderOptions {
        ModelRenderOptions {
            lighting: self.lighting,
            light_frame: self.light_frame,
            shadows: self.shadows,
            ambient_occlusion: self.occlusion,
            light_sense: self.light_sense,
            shadow_light: self.shadow_light,
            ..ModelRenderOptions::default()
        }
    }
}

/// The shadow and occlusion one frame is drawn with; `None` where switched
/// off or the scheme is unlit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Effects {
    pub(super) shadow: Option<FrameShadow>,
    pub(super) occlusion: Option<FrameAmbientOcclusion>,
}

impl Effects {
    /// The effects `options` gives a model of `bounds` from `camera` in a
    /// `size` picture: the same choice the CPU renderer makes inside.
    pub(super) fn new(
        camera: &Camera,
        options: &ModelRenderOptions,
        bounds: &Bounds,
        size: [u32; 2],
    ) -> Result<Self, RenderError> {
        Ok(Self {
            shadow: FrameShadow::new(camera, options, bounds)?,
            occlusion: FrameAmbientOcclusion::new(camera, options, bounds, size)?,
        })
    }

    /// The render trace's fields: whether each is drawn, the casting
    /// light (−1 for a dim key: the view's or the sun), the caster, and
    /// `shadow-map`, a fingerprint of the model-to-map matrix: the same
    /// value at two eyes means the shadow lies in the same model place.
    pub(super) fn trace(&self, options: &ModelRenderOptions) -> String {
        let light = self.shadow.map_or_else(String::new, |s| {
            format!(
                " shadow-light={} shadow-map={:016x}", // ui-text-exempt: diagnostic trace, never displayed
                s.light,
                fingerprint(&s.matrix)
            )
        });
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " shadows={}{light} ao={} sense={} caster={}",
            u8::from(self.shadow.is_some()),
            u8::from(self.occlusion.is_some()),
            sense_word(options.light_sense),
            caster_word(options.shadow_light)
        )
    }
}

impl ModelView {
    /// The look of the live picture.
    pub(super) fn look(&self) -> Look {
        let (lighting, light_frame) = self.lighting();
        Look {
            lighting,
            light_frame,
            shadows: self.shadows,
            occlusion: self.occlusion,
            light_sense: self.light_sense,
            shadow_light: if self.sun {
                ShadowLight::Sun {
                    up: self.upright.up.engine(),
                }
            } else {
                ShadowLight::FollowsScheme
            },
        }
    }

    /// The options menu's shadow and occlusion switches, the light sense
    /// while shadows are on, and what the picture shows of them.
    pub(super) fn effects_control(&mut self, ui: &mut Ui) {
        let shadows = ui
            .checkbox(&mut self.shadows, m::shadows())
            .on_hover_text(m::shadows_tooltip());
        crate::diag::ui_rect_visible(REGION_SHADOWS, shadows.rect, ui.clip_rect());
        if self.shadows {
            ui.horizontal_wrapped(|ui| self.shadow_light_combo(ui));
            if !self.sun {
                ui.horizontal_wrapped(|ui| self.light_sense_combo(ui));
            }
        }
        let occlusion = ui
            .checkbox(&mut self.occlusion, m::occlusion())
            .on_hover_text(m::occlusion_tooltip());
        crate::diag::ui_rect_visible(REGION_OCCLUSION, occlusion.rect, ui.clip_rect());
        if let Some(said) = self.effects_said() {
            ui.weak(said);
        }
    }

    fn shadow_light_combo(&mut self, ui: &mut Ui) {
        ui.label(m::shadow_light_label());
        let combo = egui::ComboBox::from_id_salt(REGION_SHADOW_LIGHT)
            .selected_text(m::shadow_light(self.sun))
            .show_ui(ui, |ui| {
                for sun in [true, false] {
                    let item = ui.selectable_value(&mut self.sun, sun, m::shadow_light(sun));
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!(
                        "{REGION_SHADOW_LIGHT}.{}",
                        if sun { "sun" } else { "scheme" }
                    );
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_SHADOW_LIGHT, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(m::shadow_light_tooltip());
    }

    fn light_sense_combo(&mut self, ui: &mut Ui) {
        ui.label(m::light_sense_label());
        let combo = egui::ComboBox::from_id_salt(REGION_LIGHT_SENSE)
            .selected_text(m::light_sense(self.light_sense == LightSense::Emitted))
            .show_ui(ui, |ui| {
                for sense in [LightSense::TowardsLight, LightSense::Emitted] {
                    let item = ui.selectable_value(
                        &mut self.light_sense,
                        sense,
                        m::light_sense(sense == LightSense::Emitted),
                    );
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_LIGHT_SENSE}.{}", sense_word(sense));
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_LIGHT_SENSE, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(m::light_sense_tooltip());
    }

    /// Which light casts, or why nothing is drawn, and that the standard
    /// defines neither effect; `None` with both switched off.
    fn effects_said(&self) -> Option<String> {
        if !self.shadows && !self.occlusion {
            return None;
        }
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        let aspect = f64::from(w.max(1)) / f64::from(h.max(1));
        let camera = self.view_camera(aspect).ok()?;
        let options = self.look().options();
        let effects = Effects::new(&camera, &options, &self.bounds, [w.max(1), h.max(1)]).ok()?;
        let cast = match effects.shadow {
            None => m::ShadowCast::None,
            Some(_) if self.sun => m::ShadowCast::Sun,
            Some(s) if s.light >= 0 => m::ShadowCast::SchemeLight,
            Some(_) => m::ShadowCast::ViewKey,
        };
        let drawn = effects.shadow.is_some() || effects.occlusion.is_some();
        Some(m::effects_said(cast, drawn))
    }
}

/// FNV-1a over `matrix`'s bits.
fn fingerprint(matrix: &[f32; 16]) -> u64 {
    matrix.iter().fold(0xcbf2_9ce4_8422_2325, |h, v| {
        (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The trace's word for the shadow caster.
fn caster_word(light: ShadowLight) -> &'static str {
    match light {
        ShadowLight::Sun { .. } => "sun", // ui-text-exempt: trace value, never displayed
        _ => "scheme",                    // ui-text-exempt: trace value, never displayed
    }
}

/// The trace's word for `sense`.
fn sense_word(sense: LightSense) -> &'static str {
    match sense {
        LightSense::Emitted => "emitted", // ui-text-exempt: trace value, never displayed
        _ => "towards",                   // ui-text-exempt: trace value, never displayed
    }
}
