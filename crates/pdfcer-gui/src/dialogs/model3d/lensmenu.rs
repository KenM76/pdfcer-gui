//! The lens rows on the options menu's View tab, kept between sessions, and
//! the line under the picture naming the lens it is drawn through. The
//! picture itself carries no mark of the lens.

use egui::Ui;
use pdfcer_3d::Projection;

use super::ModelView;
use super::lens::{AUTO_RANGE, CURVES, KINDS, LensKind};
use crate::text::model3dlens as l;

/// The lens drop-down's published region; each item adds `.flat`,
/// `.fisheye` or `.auto`.
pub const REGION_LENS: &str = "model3d.lens"; // ui-text-exempt: trace region name, never displayed
/// The curve drop-down's published region; each item adds `.0` or `.1`.
pub const REGION_CURVE: &str = "model3d.lens.curve"; // ui-text-exempt: trace region name, never displayed
/// Auto's change-point slider's published region.
pub const REGION_AUTO_AT: &str = "model3d.lens.auto-at"; // ui-text-exempt: trace region name, never displayed
/// Each lens item's region word, in [`KINDS`]' order.
const KIND_WORDS: [&str; 3] = ["flat", "fisheye", "auto"]; // ui-text-exempt: trace region names, never displayed

impl ModelView {
    /// Lens, then Curve where a fisheye can be drawn, then Auto's change
    /// point. Greyed, and saying why, while orbiting without perspective.
    pub(super) fn lens_control(&mut self, ui: &mut Ui) {
        let flat_only = self.nav.walk.is_none() && !self.orbit.perspective;
        let before = self.lens;
        ui.add_enabled_ui(!flat_only, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(l::lens_label()).on_hover_text(l::lens_tooltip());
                let names = l::lens_names();
                let at = KINDS.iter().position(|k| *k == self.lens.kind).unwrap_or(0);
                let combo = egui::ComboBox::from_id_salt(REGION_LENS)
                    .selected_text(names[at])
                    .show_ui(ui, |ui| {
                        for ((kind, name), word) in KINDS.into_iter().zip(names).zip(KIND_WORDS) {
                            let item = ui.selectable_value(&mut self.lens.kind, kind, name);
                            // ui-text-exempt: trace region name, never displayed
                            let region = format!("{REGION_LENS}.{word}");
                            crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                        }
                    });
                crate::diag::ui_rect_visible(REGION_LENS, combo.response.rect, ui.clip_rect());
                combo
                    .response
                    .on_hover_text(l::lens_tooltip())
                    .on_disabled_hover_text(l::lens_orthographic_tooltip());
            });
            if self.lens.kind != LensKind::Flat {
                ui.horizontal_wrapped(|ui| self.curve_combo(ui));
            }
            if self.lens.kind == LensKind::Auto {
                ui.horizontal_wrapped(|ui| {
                    ui.label(l::auto_at_label())
                        .on_hover_text(l::auto_at_tooltip());
                    let slider = ui.add(
                        egui::Slider::new(&mut self.lens.auto_at, AUTO_RANGE)
                            .step_by(1.0)
                            .suffix(l::degrees_suffix()),
                    );
                    crate::diag::ui_rect_visible(REGION_AUTO_AT, slider.rect, ui.clip_rect());
                    slider.on_hover_text(l::auto_at_tooltip());
                });
            }
        });
        if self.lens != before {
            self.lens_changed();
        }
    }

    fn curve_combo(&mut self, ui: &mut Ui) {
        ui.label(l::curve_label()).on_hover_text(l::curve_tooltip());
        let names = l::curve_names();
        let at = CURVES
            .iter()
            .position(|c| *c == self.lens.curve)
            .unwrap_or(0);
        let combo = egui::ComboBox::from_id_salt(REGION_CURVE)
            .selected_text(names[at])
            .show_ui(ui, |ui| {
                for (i, (curve, name)) in CURVES.into_iter().zip(names).enumerate() {
                    let item = ui.selectable_value(&mut self.lens.curve, curve, name);
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_CURVE}.{i}");
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                }
            });
        crate::diag::ui_rect_visible(REGION_CURVE, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(l::curve_tooltip());
    }

    /// A walk zoomed out past the new lens's widest comes back to it, so
    /// zooming in answers at once.
    fn lens_changed(&mut self) {
        let lens = self.lens;
        if let Some(walk) = &mut self.nav.walk {
            walk.zoom_by(1.0, &lens);
        }
    }

    /// The line naming the lens the picture is drawn through; `None` for a
    /// flat lens chosen, or a view with no lens.
    pub(super) fn lens_said(&self) -> Option<String> {
        if self.lens.kind == LensKind::Flat {
            return None;
        }
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        let camera = self
            .view_camera(f64::from(w.max(1)) / f64::from(h.max(1)))
            .ok()?;
        match camera.projection {
            Projection::Fisheye { fov, mapping } => {
                let at = CURVES.iter().position(|c| *c == mapping).unwrap_or(0);
                Some(l::drawn_fisheye(l::curve_names()[at], fov))
            }
            Projection::Perspective { .. } => Some(l::auto_flat(self.lens.auto_at)),
            Projection::Orthographic { .. } => None,
        }
    }
}
