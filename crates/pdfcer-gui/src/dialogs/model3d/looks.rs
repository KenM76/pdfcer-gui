//! The engine's drawing choices beyond the lighting scheme and the shadow
//! switch: the render mode (the view's own, or one picked), which lights
//! cast, how see-through parts cast, whether the model's lamps cast, a
//! camera light beside the model's own lights, and how stored shininess is
//! read. Each sits in the options menu; the model's own lights, the lights
//! and casters past the engine's limits, and why the picture is drawn
//! without the graphics card are said under the picture.

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    AssembledModel, DEFAULT_SHADOW_CUTOFF, FrameExtraShadows, FrameLights, LightingScheme,
    ModelLightKind, RenderMode, RenderModeKind, ShadowAlpha, ShadowCasters, ShininessScale,
};

use super::ModelView;
use crate::text::model3dlook as k;
use k::{Highlights, SeeThrough};

/// The render mode drop-down; each item adds `.<subtype>`, or `.view`.
pub const REGION_MODE: &str = "model3d.render_mode"; // ui-text-exempt: trace region name, never displayed
/// The drop-down for which lights cast; items add `.every` or `.key`.
pub const REGION_CASTERS: &str = "model3d.casters"; // ui-text-exempt: trace region name, never displayed
/// The drop-down for how see-through parts cast; items add `.tinted`,
/// `.cutout` or `.none`.
pub const REGION_SEE_THROUGH: &str = "model3d.see_through"; // ui-text-exempt: trace region name, never displayed
/// The lamp shadow switch.
pub const REGION_LAMP_SHADOWS: &str = "model3d.lamp_shadows"; // ui-text-exempt: trace region name, never displayed
/// The camera light switch.
pub const REGION_HEADLIGHT: &str = "model3d.headlight"; // ui-text-exempt: trace region name, never displayed
/// The highlight reading drop-down.
pub const REGION_SHININESS: &str = "model3d.shininess"; // ui-text-exempt: trace region name, never displayed

/// The choices, each defaulting to the engine's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Extras {
    /// The mode picked; `None` draws the view's.
    pub(super) mode: Option<RenderModeKind>,
    pub(super) every_light: bool,
    pub(super) see_through: SeeThrough,
    pub(super) lamp_shadows: bool,
    pub(super) headlight: bool,
    pub(super) highlights: Highlights,
}

impl Default for Extras {
    fn default() -> Self {
        let d = ModelRenderOptions::default();
        Self {
            mode: None,
            every_light: d.shadow_casters == ShadowCasters::EveryLight,
            see_through: SeeThrough::Tinted,
            lamp_shadows: d.lamp_shadows,
            headlight: d.artwork_headlight,
            highlights: Highlights::Auto,
        }
    }
}

impl Extras {
    /// Write these choices into `options`, drawing `view_mode` unless a
    /// mode is picked.
    pub(super) fn apply(self, view_mode: Option<RenderMode>, options: &mut ModelRenderOptions) {
        options.render_mode = self
            .mode
            .map(RenderMode::new)
            .or(view_mode)
            .unwrap_or_default();
        options.shadow_casters = if self.every_light {
            ShadowCasters::EveryLight
        } else {
            ShadowCasters::KeyLight
        };
        options.shadow_alpha = match self.see_through {
            SeeThrough::Tinted => ShadowAlpha::Transmit,
            SeeThrough::Cutout => ShadowAlpha::Cutout {
                cutoff: DEFAULT_SHADOW_CUTOFF,
            },
            SeeThrough::None => ShadowAlpha::OpaqueOnly,
        };
        options.lamp_shadows = self.lamp_shadows;
        options.artwork_headlight = self.headlight;
        options.shininess = match self.highlights {
            Highlights::Auto => ShininessScale::Auto,
            Highlights::Exponent => ShininessScale::Exponent,
            Highlights::Fraction => ShininessScale::Fraction,
        };
    }
}

/// The render trace's fields for these options.
pub(super) fn trace(options: &ModelRenderOptions) -> String {
    let casters = match options.shadow_casters {
        ShadowCasters::KeyLight => "key", // ui-text-exempt: trace value, never displayed
        _ => "every",                     // ui-text-exempt: trace value, never displayed
    };
    let see_through = match options.shadow_alpha {
        ShadowAlpha::OpaqueOnly => "none", // ui-text-exempt: trace value, never displayed
        ShadowAlpha::Cutout { .. } => "cutout", // ui-text-exempt: trace value, never displayed
        _ => "tinted",                     // ui-text-exempt: trace value, never displayed
    };
    format!(
        // ui-text-exempt: diagnostic trace, never displayed
        " render-mode={} casters={casters} see-through={see_through} lamp-shadows={} headlight={}",
        options.render_mode.kind.as_name(),
        u8::from(options.lamp_shadows),
        u8::from(options.artwork_headlight)
    )
}

/// How many of `model`'s lights are lamps (point or spot), directional and
/// ambient.
pub(super) fn light_kinds(model: &AssembledModel) -> [usize; 3] {
    let mut n = [0; 3];
    for light in &model.lights {
        let slot = match light.kind {
            ModelLightKind::Point { .. } | ModelLightKind::Spot { .. } => 0,
            ModelLightKind::Directional { .. } => 1,
            _ => 2,
        };
        if let Some(c) = n.get_mut(slot) {
            *c += 1;
        }
    }
    n
}

impl ModelView {
    /// The render mode the view on screen asks for.
    pub(super) fn view_mode(&self) -> Option<RenderMode> {
        self.aimed
            .as_ref()
            .and_then(|v| v.render_mode)
            .map(Into::into)
    }

    /// The options menu's render mode drop-down, and the shadow, lamp,
    /// camera light and highlight choices that apply to this model.
    pub(super) fn looks_control(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| self.mode_combo(ui));
        let [lamps, _, _] = light_kinds(&self.model.drawn);
        if self.shadows {
            if !self.sun {
                ui.horizontal_wrapped(|ui| self.casters_combo(ui));
            }
            ui.horizontal_wrapped(|ui| self.see_through_combo(ui));
            if lamps > 0 {
                let r = ui
                    .checkbox(&mut self.extras.lamp_shadows, k::lamp_shadows())
                    .on_hover_text(k::lamp_shadows_tooltip());
                crate::diag::ui_rect_visible(REGION_LAMP_SHADOWS, r.rect, ui.clip_rect());
            }
        }
        if !self.model.drawn.lights.is_empty() {
            let r = ui
                .checkbox(&mut self.extras.headlight, k::headlight())
                .on_hover_text(k::headlight_tooltip());
            crate::diag::ui_rect_visible(REGION_HEADLIGHT, r.rect, ui.clip_rect());
        }
        if self.model.drawn.finishes.iter().any(Option::is_some) {
            ui.horizontal_wrapped(|ui| self.highlights_combo(ui));
        }
    }

    fn mode_combo(&mut self, ui: &mut Ui) {
        ui.label(k::mode_label());
        let from_view = k::mode_from_view(self.view_mode().unwrap_or_default().kind.as_name());
        let shown = self
            .extras
            .mode
            .map_or_else(|| from_view.clone(), |m| k::mode(m.as_name()).to_owned());
        let combo = egui::ComboBox::from_id_salt(REGION_MODE)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                let item = ui.selectable_value(&mut self.extras.mode, None, from_view);
                // ui-text-exempt: trace region name, never displayed
                let region = format!("{REGION_MODE}.view");
                crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                for kind in RenderModeKind::ALL {
                    let word = k::mode(kind.as_name());
                    let item = ui.selectable_value(&mut self.extras.mode, Some(kind), word);
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_MODE}.{}", kind.as_name());
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_MODE, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(k::mode_tooltip());
    }

    fn casters_combo(&mut self, ui: &mut Ui) {
        ui.label(k::casters_label());
        let combo = egui::ComboBox::from_id_salt(REGION_CASTERS)
            .selected_text(k::casters(self.extras.every_light))
            .show_ui(ui, |ui| {
                for (every, word) in [(true, "every"), (false, "key")] {
                    let item =
                        ui.selectable_value(&mut self.extras.every_light, every, k::casters(every));
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_CASTERS}.{word}");
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_CASTERS, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(k::casters_tooltip());
    }

    fn see_through_combo(&mut self, ui: &mut Ui) {
        ui.label(k::alpha_label());
        let combo = egui::ComboBox::from_id_salt(REGION_SEE_THROUGH)
            .selected_text(k::alpha(self.extras.see_through))
            .show_ui(ui, |ui| {
                for (way, word) in [
                    (SeeThrough::Tinted, "tinted"),
                    (SeeThrough::Cutout, "cutout"),
                    (SeeThrough::None, "none"),
                ] {
                    let item =
                        ui.selectable_value(&mut self.extras.see_through, way, k::alpha(way));
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_SEE_THROUGH}.{word}");
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_SEE_THROUGH, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(k::alpha_tooltip());
    }

    fn highlights_combo(&mut self, ui: &mut Ui) {
        ui.label(k::shininess_label());
        let combo = egui::ComboBox::from_id_salt(REGION_SHININESS)
            .selected_text(k::shininess(self.extras.highlights))
            .show_ui(ui, |ui| {
                for way in [Highlights::Auto, Highlights::Exponent, Highlights::Fraction] {
                    ui.selectable_value(&mut self.extras.highlights, way, k::shininess(way));
                }
            });
        crate::diag::ui_rect_visible(REGION_SHININESS, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(k::shininess_tooltip());
    }

    /// Under the picture: the model's own lights, the lights and casters
    /// past the engine's limits, and why the graphics card is not drawing.
    pub(super) fn looks_notes(&self, ui: &mut Ui) {
        let d = &self.model.drawn;
        let options = self.look().options();
        if !d.lights.is_empty() {
            let [lamps, directional, ambient] = light_kinds(d);
            let drawn = options.lighting == LightingScheme::Artwork;
            ui.small(k::model_lights(lamps, directional, ambient, drawn));
        }
        if let Some(camera) = self.notes_camera() {
            let lights = FrameLights::for_model(&camera, &options, &d.lights);
            if let Some(n) = lights.ok().map(|l| l.dropped).filter(|&n| n > 0) {
                ui.small(k::lights_dropped(n));
            }
            let extras = FrameExtraShadows::for_model(&camera, &options, &self.bounds, &d.lights);
            if let Some(n) = extras.ok().flatten().map(|e| e.dropped).filter(|&n| n > 0) {
                ui.small(k::casters_dropped(n));
            }
        }
        if let Some(reason) = self.gpu.declined() {
            ui.small(k::drawn_on_processor(k::processor_reason(reason)));
        }
    }

    /// The camera of the picture last drawn.
    fn notes_camera(&self) -> Option<pdfcer_3d::Camera> {
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        self.view_camera(f64::from(w.max(1)) / f64::from(h.max(1)))
            .ok()
    }
}

/// The render trace's field for a frame the card declined.
pub(super) fn declined_word(reason: k::ProcessorReason) -> &'static str {
    match reason {
        k::ProcessorReason::Lines => " gpu-declined=lines", // ui-text-exempt: trace value, never displayed
        k::ProcessorReason::LampShadows => " gpu-declined=lamp-shadows", // ui-text-exempt: trace value, never displayed
        k::ProcessorReason::ExtraCasters => " gpu-declined=extra-casters", // ui-text-exempt: trace value, never displayed
        k::ProcessorReason::SeeThrough => " gpu-declined=see-through", // ui-text-exempt: trace value, never displayed
    }
}
