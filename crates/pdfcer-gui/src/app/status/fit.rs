//! # `status::fit` — the bar's four named zoom levels
//!
//!
//! ## The one thing to read before changing anything here
//!
//! The bar's right-hand cluster is laid out **right to left**, so within this
//! group the control added FIRST is drawn RIGHTMOST. The screen reads
//! `Actual size · Fit width · Fit height · Fit page`; the calls below run in
//! the reverse of that. Getting it backwards does not break anything — it
//! silently reorders four controls an operator has learned the positions of.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/fit.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::status as t;
use crate::viewer::FitMode;
use pdfcer_gui_base::icons::{self, Icon};

use super::REGION_FIT;

/// `Actual size · Fit width · Fit page`, mirroring View ▸ Zoom under P1a.
pub(super) fn group(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let fit = doc.view.fit;
    let rect = ui
        .scope(|ui| {
            // Right-to-left: added first is drawn rightmost, so the screen
            // reads `Actual size · Fit width · Fit height · Fit page`.
            if ui
                .add(icons::labelled(
                    ui,
                    Icon::FitPage,
                    t::fit_page(),
                    fit == FitMode::Page,
                ))
                .on_hover_text(t::fit_page_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Page));
            }
            // Between Fit page and Fit width on screen — O29. Added
            // second here because the layout is right-to-left, so the bar
            // reads `Actual size · Fit width · Fit height · Fit page` and the
            // two single-axis fits sit beside each other rather than with
            // Fit page wedged between them.
            if ui
                .add(icons::labelled(
                    ui,
                    Icon::FitHeight,
                    t::fit_height(),
                    fit == FitMode::Height,
                ))
                .on_hover_text(t::fit_height_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Height));
            }
            if ui
                .add(icons::labelled(
                    ui,
                    Icon::FitWidth,
                    t::fit_width(),
                    fit == FitMode::Width,
                ))
                .on_hover_text(t::fit_width_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Width));
            }
            // Raises exactly what the ribbon's `view.zoom_actual` raises,
            // including its defect. See the module docs: the fix is a new
            // action variant, not a divergent mirror.
            if ui
                .add(icons::labelled(
                    ui,
                    Icon::ActualSize,
                    t::fit_actual_size(),
                    false,
                ))
                .on_hover_text(t::fit_actual_size_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::None));
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_FIT, rect);
}
