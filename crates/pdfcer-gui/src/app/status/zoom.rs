//! # `app::status::zoom` — the zoom controls, and the maximum behind them
//!
//! `− ⟨percent⟩ +` on the status bar, and — since 2026-08-22 — the
//! maximum-zoom popup the readout opens.
//!
//! ## Why this is a file
//!
//! R2's 1,500-line ceiling forced the split when the popup landed, and as
//! with [`super::page_box`], [`super::notes`], [`super::decline`] and
//! [`super::filter`] before it, the forced seam is a real one: this group is
//! now the only part of the bar that both *reports* a value and *edits a
//! preference*, which is a different subject from the readouts around it.

use egui::Vec2;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::status as t;

use super::{ROW_HEIGHT_PTS, ZOOM_READOUT_WIDTH_PTS};
/// `−  ⟨percent⟩  +`.
///
/// The readout is a label rather than a field: there is no action that sets
/// a zoom to a named value (see [`crate::text::status::zoom_percent`]), and
/// a text box in front of nothing is a placeholder. It is given a fixed
/// width so that stepping from `100%` to `75%` does not move the − button
/// out from under the operator's pointer.
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
            // ★★ The readout is a BUTTON now — O24. Same fixed width, so
            // nothing on the bar moves; `Button::frame(false)` keeps it
            // looking like the readout it has always been rather than
            // growing a border the operator has to learn.
            //
            // ★ It is still not editable, and the reason `page_box` gives for
            // being a `TextEdit` is why: a page NUMBER is a value you type,
            // where a zoom is a value you step. This opens a list of
            // maximums; it does not invite a percentage.
            let readout = ui
                .add_sized(
                    Vec2::new(ZOOM_READOUT_WIDTH_PTS, ROW_HEIGHT_PTS),
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
