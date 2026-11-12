//! The figure's controls in the options' Moving tab, while walking is seen
//! from behind: its five proportions, its two shadows, and *As made*.

use egui::{Slider, Ui};

use super::ModelView;
use super::figure::{MEASURE, Measures};
use crate::text::panels::modelfigure as t;

/// The shadow-underneath switch.
pub const REGION_FIGURE_BLOB: &str = "model3d.figure.blob"; // ui-text-exempt: trace region name, never displayed
/// The cast-shadow switch.
pub const REGION_FIGURE_CAST: &str = "model3d.figure.cast"; // ui-text-exempt: trace region name, never displayed
/// The legs' proportion.
pub const REGION_FIGURE_LEGS: &str = "model3d.figure.legs"; // ui-text-exempt: trace region name, never displayed

impl ModelView {
    /// The figure's controls; nothing unless walking is seen from behind.
    pub(super) fn figure_controls(&mut self, ui: &mut Ui) {
        if !(self.nav.prefs.behind && self.nav.walk.is_some()) {
            return;
        }
        ui.separator();
        ui.label(t::heading());
        let before = self.nav.prefs.figure;
        let m = &mut self.nav.prefs.figure;
        let rows: [(&str, &mut f64, Option<&str>); 5] = [
            (t::head(), &mut m.head, None),
            (t::shoulders(), &mut m.shoulders, None),
            (t::arms(), &mut m.arms, None),
            (t::legs(), &mut m.legs, Some(REGION_FIGURE_LEGS)),
            (t::build(), &mut m.build, None),
        ];
        for (label, value, region) in rows {
            let slider = ui
                .add(
                    Slider::new(value, MEASURE)
                        .text(label)
                        .custom_formatter(|v, _| t::percent(v)),
                )
                .on_hover_text(t::measure_tooltip());
            if let Some(region) = region {
                crate::diag::ui_rect_visible(region, slider.rect, ui.clip_rect());
            }
        }
        ui.horizontal_wrapped(|ui| {
            let blob = ui
                .checkbox(&mut m.blob, t::blob())
                .on_hover_text(t::blob_tooltip());
            crate::diag::ui_rect_visible(REGION_FIGURE_BLOB, blob.rect, ui.clip_rect());
            let cast = ui
                .checkbox(&mut m.cast, t::cast())
                .on_hover_text(t::cast_tooltip());
            crate::diag::ui_rect_visible(REGION_FIGURE_CAST, cast.rect, ui.clip_rect());
            if ui
                .button(t::reset())
                .on_hover_text(t::reset_tooltip())
                .clicked()
            {
                *m = m.as_made();
            }
        });
        if self.nav.prefs.figure != before {
            self.nav.store(ui.ctx());
            trace_figure(&self.nav.prefs.figure);
        }
    }
}

fn trace_figure(m: &Measures) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "model-figure-prefs head={:.2} shoulders={:.2} arms={:.2} legs={:.2} build={:.2} blob={} cast={}",
            m.head, m.shoulders, m.arms, m.legs, m.build, m.blob, m.cast
        )
    });
}
