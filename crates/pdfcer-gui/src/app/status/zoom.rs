//! # `app::status::zoom` — the zoom controls, and the maximum behind them
//!
//!
//! ## Why this is a file
//!
//! R2's 1,500-line ceiling forced the split when the popup landed, and as
//! with [`super::page_box`], [`super::notes`], [`super::decline`] and
//! [`super::filter`] before it, the forced seam is a real one: this group is
//! now the only part of the bar that both *reports* a value and *edits a
//! preference*, which is a different subject from the readouts around it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/zoom.md`.

use egui::Vec2;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::status as t;

use super::{ROW_HEIGHT_PTS, ZOOM_READOUT_WIDTH_PTS};

/// How wide the readout must be so that stepping the zoom never moves the
/// buttons beside it.
fn readout_width(ui: &egui::Ui, max_zoom_percent: f32) -> f32 {
    let widest = t::zoom_percent(f64::from(max_zoom_percent));
    let galley = ui.painter().layout_no_wrap(
        widest,
        egui::TextStyle::Button.resolve(ui.style()),
        egui::Color32::PLACEHOLDER,
    );
    (galley.size().x + 2.0).max(ZOOM_READOUT_WIDTH_PTS)
}
/// `−  ⟨percent⟩  +`.
pub(super) fn group(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    max_zoom_percent: &mut f32,
    actions: &mut Vec<Action>,
) {
    let percent = doc.view.zoom_percent();
    let rect = ui
        .scope(|ui| {
            // Right-to-left: added first is drawn rightmost, so the screen
            // reads `− ⟨percent⟩ +`.
            if ui
                .button(t::zoom_in())
                .on_hover_text(t::zoom_in_tooltip())
                .clicked()
            {
                actions.push(Action::ZoomIn);
            }
            // The readout is a BUTTON now — O24. Same fixed width, so
            // nothing on the bar moves; `Button::frame(false)` keeps it
            // looking like the readout it has always been rather than
            // growing a border the operator has to learn.
            //
            // It is still not editable, and the reason `page_box` gives for
            // being a `TextEdit` is why: a page NUMBER is a value you type,
            // where a zoom is a value you step. This opens a list of
            // maximums; it does not invite a percentage.
            let readout = ui
                .add_sized(
                    Vec2::new(readout_width(ui, *max_zoom_percent), ROW_HEIGHT_PTS),
                    egui::Button::new(t::zoom_percent(percent)).frame(false),
                )
                .on_hover_text(crate::text::maxzoom::readout_tooltip());
            egui::Popup::menu(&readout)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                .show(|ui| super::maxzoom::popup(ui, max_zoom_percent));
            if ui
                .button(t::zoom_out())
                .on_hover_text(t::zoom_out_tooltip())
                .clicked()
            {
                actions.push(Action::ZoomOut);
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(super::REGION_ZOOM, rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The premise the old constant asserted, measured — and it is false.
    #[test]
    fn the_readout_can_be_asked_to_draw_far_more_than_four_characters() {
        let widest = t::zoom_percent(f64::from(crate::app::prefs::MAX_MAX_ZOOM_PERCENT));
        assert!(
            widest.chars().count() > 4,
            "the reserve was sized for four characters; the ceiling can produce {:?} ({} of them)",
            widest,
            widest.chars().count()
        );
        // And the old ceiling really was four, which is why the constant was
        // right when it was written. Both halves matter: the constant was not
        // careless, it was overtaken.
        assert_eq!(t::zoom_percent(800.0).chars().count(), 4);
    }

    /// The measured reserve tracks the ceiling, and never drops below the floor.
    #[test]
    fn the_reserve_grows_with_the_ceiling_and_never_shrinks_below_the_floor() {
        let ctx = egui::Context::default();
        let mut narrow = 0.0_f32;
        let mut wide = 0.0_f32;
        // Two frames: the first builds the font atlas, the second measures
        // against it. A galley measured on the very first frame of a fresh
        // `Context` is laid out before fonts are ready, which would make this
        // test assert about a placeholder rather than about text.
        for _ in 0..2 {
            let _ = ctx.run_ui(Default::default(), |ui| {
                narrow = readout_width(ui, 800.0);
                wide = readout_width(ui, crate::app::prefs::MAX_MAX_ZOOM_PERCENT);
            });
        }
        assert!(
            narrow >= ZOOM_READOUT_WIDTH_PTS,
            "the floor must hold at the bottom of the range: {narrow} < {ZOOM_READOUT_WIDTH_PTS}"
        );
        assert!(
            wide > narrow,
            "a ceiling of a trillion percent must reserve more room than 800 %: {wide} vs {narrow}"
        );
    }
}
