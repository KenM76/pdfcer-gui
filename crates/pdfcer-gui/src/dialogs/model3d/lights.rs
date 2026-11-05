//! How the live picture is lit: the scheme the operator picks, else the one
//! the view on screen asks for (the file's opening view or a saved view
//! chosen from its list), else the engine's default; and whether the lights
//! turn with the view. The scheme and where it came from are said under the
//! drop-down, never drawn into the picture.

use egui::Ui;
use pdfcer_3d::{LightFrame, LightingScheme};
use pdfcer_core::threed::ThreeDLighting;

use super::ModelView;
use crate::text::model3d as m;

/// The options menu's lighting drop-down.
pub const REGION_LIGHTING: &str = "model3d.lighting"; // ui-text-exempt: trace region name, never displayed
/// Its switch for lights that turn with the view.
pub const REGION_LIGHT_FRAME: &str = "model3d.light_frame"; // ui-text-exempt: trace region name, never displayed

/// Where the scheme lighting the picture came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LitBy {
    /// Picked in the viewer.
    Chosen,
    /// The view on screen asks for it in the file.
    File,
    /// Neither asks; the engine's default.
    Default,
}

impl LitBy {
    /// The trace's word for it.
    pub(super) fn word(self) -> &'static str {
        match self {
            LitBy::Chosen => "chosen", // ui-text-exempt: trace value, never displayed
            LitBy::File => "file",     // ui-text-exempt: trace value, never displayed
            LitBy::Default => "default", // ui-text-exempt: trace value, never displayed
        }
    }
}

impl ModelView {
    /// The scheme the live picture is drawn with, and where it came from.
    pub(super) fn lit_by(&self) -> (ThreeDLighting, LitBy) {
        if let Some(scheme) = self.view_lighting {
            return (scheme, LitBy::Chosen);
        }
        match self.aimed.as_ref().and_then(|v| v.lighting) {
            Some(scheme) => (scheme, LitBy::File),
            None => (ThreeDLighting::Artwork, LitBy::Default),
        }
    }

    /// The renderer's lighting and light frame for the live picture.
    pub(super) fn lighting(&self) -> (LightingScheme, LightFrame) {
        let frame = if self.lights_turn {
            LightFrame::Camera
        } else {
            LightFrame::World
        };
        (self.lit_by().0.into(), frame)
    }

    /// The lighting drop-down, under `region`; the same choice lights the
    /// live picture and is written into saved views.
    pub(super) fn lighting_combo(&mut self, ui: &mut Ui, region: &str) {
        let shown = self
            .view_lighting
            .map_or_else(m::lighting_unset, m::lighting);
        let combo = egui::ComboBox::from_id_salt(region)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.view_lighting, None, m::lighting_unset());
                for scheme in m::lighting_schemes() {
                    let item = ui.selectable_value(
                        &mut self.view_lighting,
                        Some(scheme),
                        m::lighting(scheme),
                    );
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{region}.{}", scheme_word(scheme));
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(region, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(m::views_lighting_tooltip());
    }

    /// The render trace's lighting fields for `scheme`, the scheme drawn.
    pub(super) fn lighting_trace(&self, scheme: LightingScheme) -> String {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " lighting={} lit-by={} lights-turn={}",
            scheme.name(),
            self.lit_by().1.word(),
            u8::from(self.lights_turn)
        )
    }

    /// The options menu's lighting group: the drop-down, the switch, and
    /// which scheme lights the picture and why.
    pub(super) fn lighting_control(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(m::lighting_label());
            self.lighting_combo(ui, REGION_LIGHTING);
        });
        self.brightness_control(ui);
        let turn = ui
            .checkbox(&mut self.lights_turn, m::lights_turn())
            .on_hover_text(m::lights_turn_tooltip());
        crate::diag::ui_rect_visible(REGION_LIGHT_FRAME, turn.rect, ui.clip_rect());
        let (scheme, by) = self.lit_by();
        ui.weak(m::lit_by(
            m::lighting(scheme),
            by == LitBy::File,
            by == LitBy::Chosen,
        ));
    }
}

/// `scheme`'s PDF name in lower case, for a region name.
fn scheme_word(scheme: ThreeDLighting) -> String {
    LightingScheme::from(scheme).name().to_ascii_lowercase()
}
