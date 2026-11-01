//! The walker's eye height in feet and inches (O356): shown with ghost mode
//! off, for a model whose file says its unit. The height is kept in
//! millimetres with the navigation switches and in the preferences file, so
//! it holds from model to model and across restarts; the model's unit turns
//! it into model units.

use egui::{DragValue, Ui};
use pdfcer_3d::ModelUnits;
use pdfcer_gui_base::units::{
    inches_from_points, mm_from_points, points_from_inches, points_from_mm,
};

use super::ModelView;
use crate::text::panels::modelnav as t;

/// The eye height row's two fields.
pub const REGION_EYE_HEIGHT: &str = "model3d.eye-height"; // ui-text-exempt: trace region name, never displayed

/// The lowest eye height accepted, inches.
const LOWEST_IN: f64 = 1.0;

impl ModelView {
    /// The eye height the operator set, in model units, when the model's
    /// unit is known.
    pub(super) fn eye_height(&self) -> Option<f64> {
        let mm = self.nav.prefs.eye_mm?;
        let units = self.model_units()?;
        Some(mm / units.mm_per_unit)
    }

    /// The model's unit, when it is a usable one.
    fn model_units(&self) -> Option<ModelUnits> {
        self.model
            .drawn
            .units
            .filter(|u| u.mm_per_unit.is_finite() && u.mm_per_unit > 0.0)
    }

    /// Feet and inches fields for the eye height, and *Auto* to go back to a
    /// share of the model's size. Nothing while orbiting, in ghost mode, or
    /// for a model with no unit.
    pub(super) fn eye_height_control(&mut self, ui: &mut Ui) {
        let (Some(walk), false, Some(units)) =
            (self.nav.walk, self.nav.prefs.ghost, self.model_units())
        else {
            return;
        };
        // Split in whole tenths of an inch, the field's step: an unrounded
        // 11.99 in would be clamped to 11.9 by the field, and the clamp
        // reads as an edit that lowers the stored height.
        let tenths = (inches_from_mm(walk.body() * units.mm_per_unit) * 10.0).round();
        let mut feet = (tenths / 120.0).floor();
        let mut inches = (tenths - feet * 120.0) / 10.0;
        ui.label(t::eye_height())
            .on_hover_text(t::eye_height_tooltip());
        let feet_field = ui.add(
            DragValue::new(&mut feet)
                .range(0.0..=999.0)
                .max_decimals(0)
                .speed(0.05)
                .suffix(t::feet_suffix()),
        );
        let inch_field = ui.add(
            DragValue::new(&mut inches)
                .range(0.0..=11.9)
                .max_decimals(1)
                .speed(0.1)
                .suffix(t::inches_suffix()),
        );
        let fields = feet_field.rect.union(inch_field.rect);
        crate::diag::ui_rect_visible(REGION_EYE_HEIGHT, fields, ui.clip_rect());
        if feet_field.changed() || inch_field.changed() {
            let wanted = (feet.round() * 12.0 + inches).max(LOWEST_IN);
            self.nav.prefs.eye_mm = Some(mm_from_inches(wanted));
        }
        let settled = [&feet_field, &inch_field]
            .iter()
            .any(|r| r.drag_stopped() || r.lost_focus());
        if settled {
            self.eye_changed = true;
            self.switched(ui.ctx());
        }
        if !units.from_cad_file {
            ui.weak(t::unit_assumed())
                .on_hover_text(t::unit_assumed_tooltip());
        }
        if self.nav.prefs.eye_mm.is_some()
            && ui
                .button(t::eye_auto())
                .on_hover_text(t::eye_auto_tooltip())
                .clicked()
        {
            self.nav.prefs.eye_mm = None;
            self.eye_changed = true;
            self.switched(ui.ctx());
        }
    }
}

fn inches_from_mm(mm: f64) -> f64 {
    inches_from_points(points_from_mm(mm))
}

fn mm_from_inches(inches: f64) -> f64 {
    mm_from_points(points_from_inches(inches))
}
