//! What the live picture is drawn with: the lighting, and the shadows and
//! ambient occlusion the engine adds although the PDF standard defines
//! neither. Their switches sit under the lighting drop-down, and which light
//! casts, or why nothing does, is said there, never drawn into the picture.

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    Bounds, Camera, FrameAmbientOcclusion, FrameShadow, LightFrame, LightSense, LightingScheme,
    RenderError,
};

use super::ModelView;
use crate::text::model3d as m;

/// The shadows switch.
pub const REGION_SHADOWS: &str = "model3d.shadows"; // ui-text-exempt: trace region name, never displayed
/// The ambient occlusion switch.
pub const REGION_OCCLUSION: &str = "model3d.occlusion"; // ui-text-exempt: trace region name, never displayed
/// The drop-down for which way a scheme's light directions point.
pub const REGION_LIGHT_SENSE: &str = "model3d.light_sense"; // ui-text-exempt: trace region name, never displayed

/// Everything the picture's look depends on besides the model and camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Look {
    pub(super) lighting: LightingScheme,
    pub(super) light_frame: LightFrame,
    pub(super) shadows: bool,
    pub(super) occlusion: bool,
    pub(super) light_sense: LightSense,
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

    /// The render trace's fields: whether each is drawn, and the casting
    /// light, −1 for the fixed key light.
    pub(super) fn trace(&self, sense: LightSense) -> String {
        let light = self
            .shadow
            .map_or_else(String::new, |s| format!(" shadow-light={}", s.light)); // ui-text-exempt: diagnostic trace, never displayed
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " shadows={}{light} ao={} sense={}",
            u8::from(self.shadow.is_some()),
            u8::from(self.occlusion.is_some()),
            sense_word(sense)
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
            ui.horizontal_wrapped(|ui| self.light_sense_combo(ui));
        }
        let occlusion = ui
            .checkbox(&mut self.occlusion, m::occlusion())
            .on_hover_text(m::occlusion_tooltip());
        crate::diag::ui_rect_visible(REGION_OCCLUSION, occlusion.rect, ui.clip_rect());
        if let Some(said) = self.effects_said() {
            ui.weak(said);
        }
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
        let cast = effects.shadow.map(|s| s.light >= 0);
        let drawn = effects.shadow.is_some() || effects.occlusion.is_some();
        Some(m::effects_said(self.shadows.then_some(cast), drawn))
    }
}

/// The trace's word for `sense`.
fn sense_word(sense: LightSense) -> &'static str {
    match sense {
        LightSense::Emitted => "emitted", // ui-text-exempt: trace value, never displayed
        _ => "towards",                   // ui-text-exempt: trace value, never displayed
    }
}
