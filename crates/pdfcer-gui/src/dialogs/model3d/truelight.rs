//! *True light level*: the live picture, the poster and the saved picture
//! drawn at an exposure fixed in lux (`pdfcer_3d::Exposure::Fixed`) instead
//! of the metered one, so a dim room draws dim beside a bright one.
//!
//! Every draw takes its lights and options from [`lit`]: the processor's
//! picture through `ExposedLighting::render`, the card's lights, shadows,
//! occlusion and bounce from the same lights and options, so the two match.
//! When the engine refuses (a light with no physical strength, a headlight,
//! another lighting scheme), the picture is drawn metered and the reasons
//! are said beside the switch and under the picture, never on it.

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    AssembledModel, Camera, ExposedLighting, Exposure, ExposureError, Image, MeterSettings,
    PreparedModel, RenderError, render_model,
};

use super::ModelView;
use crate::text::model3dlight as g;

/// The switch.
pub const REGION_TRUE_LIGHT: &str = "model3d.true_light"; // ui-text-exempt: trace region name, never displayed
/// The lux drawn as full white.
pub const REGION_WHITE_LUX: &str = "model3d.white_lux"; // ui-text-exempt: trace region name, never displayed
/// The lux a white surface is drawn full white at, by default: a well-lit
/// office.
pub(super) const DEFAULT_WHITE_LUX: f64 = 500.0;
/// The white point's range, lux.
pub(super) const WHITE_LUX_RANGE: std::ops::RangeInclusive<f64> = 1.0..=100_000.0;

/// The switch and its white point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TrueLight {
    pub on: bool,
    /// Lux drawn as full white.
    pub white_lux: f64,
}

impl Default for TrueLight {
    fn default() -> Self {
        Self {
            on: false,
            white_lux: DEFAULT_WHITE_LUX,
        }
    }
}

impl TrueLight {
    /// The exposure asked for.
    pub(super) fn exposure(self) -> Exposure {
        if self.on {
            Exposure::Fixed {
                white_lux: self.white_lux,
            }
        } else {
            Exposure::Metered
        }
    }
}

/// The lights and options one frame is drawn with, and why a fixed
/// exposure asked for was not drawn.
pub(super) struct Lit {
    pub drawn: ExposedLighting,
    pub refused: Option<ExposureError>,
}

/// `model` drawn with `options` at `exposure`, metered where the engine
/// refuses it. `None` only if the engine refuses the metered exposure too,
/// which it never does.
pub(super) fn lit(
    model: &AssembledModel,
    exposure: Exposure,
    options: &ModelRenderOptions,
) -> Option<Lit> {
    let settings = MeterSettings::default();
    match exposure.lighting(model, options, &settings) {
        Ok(drawn) => Some(Lit {
            drawn,
            refused: None,
        }),
        Err(why) => Exposure::Metered
            .lighting(model, options, &settings)
            .ok()
            .map(|drawn| Lit {
                drawn,
                refused: Some(why),
            }),
    }
}

/// `model` drawn by the processor from `camera` with `options` at
/// `exposure`, as [`lit`] lights it.
pub(super) fn render_lit(
    model: &AssembledModel,
    camera: &Camera,
    options: &ModelRenderOptions,
    exposure: Exposure,
) -> Result<Image, RenderError> {
    match lit(model, exposure, options) {
        Some(l) => l.drawn.render(&PreparedModel::new(model), model, camera),
        None => render_model(model, camera, options),
    }
}

impl Lit {
    /// The render trace's fields: `exposure=metered|fixed|refused`, the
    /// white point drawn (0 when metered) and the engine's reason and note
    /// counts.
    pub(super) fn trace(&self) -> String {
        let state = match (&self.drawn.white_lux, &self.refused) {
            (_, Some(_)) => "refused",  // ui-text-exempt: trace value, never displayed
            (Some(_), None) => "fixed", // ui-text-exempt: trace value, never displayed
            (None, None) => "metered",  // ui-text-exempt: trace value, never displayed
        };
        let reasons = match &self.refused {
            Some(ExposureError::Relative(r)) => r.len(),
            Some(_) => 1,
            None => 0,
        };
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " exposure={state} white-lux={:.1} exposure-reasons={reasons} exposure-notes={}",
            self.drawn.white_lux.unwrap_or(0.0),
            self.drawn.notes.len()
        )
    }

    /// What is said off the picture: the white point drawn and the
    /// engine's notes, or why it is not drawn; nothing when metered.
    pub(super) fn said(&self) -> Vec<String> {
        if let Some(why) = &self.refused {
            let reasons = match why {
                ExposureError::Relative(r) => join(r.iter().map(ToString::to_string)),
                other => other.to_string(),
            };
            return vec![g::true_light_refused(&reasons)];
        }
        let Some(white) = self.drawn.white_lux else {
            return Vec::new();
        };
        let mut lines = vec![g::true_light_said(white)];
        if !self.drawn.notes.is_empty() {
            let notes = join(self.drawn.notes.iter().map(ToString::to_string));
            lines.push(g::true_light_notes(&notes));
        }
        lines
    }
}

fn join(parts: impl Iterator<Item = String>) -> String {
    parts.collect::<Vec<_>>().join(g::NOTES_SEPARATOR)
}

impl ModelView {
    /// How the live picture is lit at `options`, its exposure from the look.
    pub(super) fn lit_now(&self, options: &ModelRenderOptions) -> Option<Lit> {
        lit(&self.model.drawn, self.light.true_light.exposure(), options)
    }

    /// The picture is drawn in lux now: the switch is on and the engine
    /// accepts it.
    pub(super) fn drawn_in_lux(&self) -> bool {
        self.light.true_light.on
            && self
                .lit_now(&self.look().options())
                .is_some_and(|l| l.refused.is_none())
    }

    /// The switch and its white point, for a model with lights of its own;
    /// what it draws, or why not, beneath.
    pub(super) fn true_light_control(&mut self, ui: &mut Ui) {
        if !self.lights.has_lamps() {
            return;
        }
        let before = self.light.true_light;
        let switch = ui
            .checkbox(&mut self.light.true_light.on, g::true_light())
            .on_hover_text(g::true_light_tooltip());
        crate::diag::ui_rect_visible(REGION_TRUE_LIGHT, switch.rect, ui.clip_rect());
        if self.light.true_light.on {
            ui.horizontal_wrapped(|ui| {
                ui.label(g::white_lux_label())
                    .on_hover_text(g::white_lux_tooltip());
                let field = ui.add(
                    egui::DragValue::new(&mut self.light.true_light.white_lux)
                        .range(WHITE_LUX_RANGE)
                        .speed(5.0)
                        .max_decimals(0)
                        .suffix(g::LUX_SUFFIX),
                );
                crate::diag::ui_rect_visible(REGION_WHITE_LUX, field.rect, ui.clip_rect());
                field.on_hover_text(g::white_lux_tooltip());
            });
            if let Some(lit) = self.lit_now(&self.look().options()) {
                for line in lit.said() {
                    ui.weak(line);
                }
            }
        }
        let after = self.light.true_light;
        if after != before {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-true-light on={} white-lux={:.1}",
                    u8::from(after.on),
                    after.white_lux
                )
            });
        }
    }

    /// Under the picture: the white point drawn and the engine's notes, or
    /// why the switch is not drawn.
    pub(super) fn true_light_notes(&self, ui: &mut Ui, options: &ModelRenderOptions) {
        if !self.light.true_light.on {
            return;
        }
        if let Some(lit) = self.lit_now(options) {
            for line in lit.said() {
                ui.small(line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_asks_for_the_metered_picture_and_on_for_its_white_point() {
        let mut t = TrueLight::default();
        assert_eq!(t.exposure(), Exposure::Metered);
        t.on = true;
        assert_eq!(t.exposure(), Exposure::Fixed { white_lux: 500.0 });
    }

    #[test]
    fn a_model_without_lights_is_drawn_metered_and_says_why() {
        let model = AssembledModel::default();
        let options = ModelRenderOptions::default();
        let fixed = Exposure::Fixed { white_lux: 500.0 };
        let Some(l) = lit(&model, fixed, &options) else {
            panic!("the metered fallback is always drawn");
        };
        assert!(l.refused.is_some());
        assert_eq!(l.drawn.white_lux, None);
        assert_eq!(l.drawn.options, options);
        assert!(l.trace().contains(" exposure=refused "));
        assert_eq!(l.said().len(), 1);
    }

    #[test]
    fn metered_says_nothing() {
        let model = AssembledModel::default();
        let options = ModelRenderOptions::default();
        let Some(l) = lit(&model, Exposure::Metered, &options) else {
            panic!("the metered picture is always drawn");
        };
        assert!(l.said().is_empty());
        assert!(l.trace().contains(" exposure=metered white-lux=0.0 "));
    }
}
