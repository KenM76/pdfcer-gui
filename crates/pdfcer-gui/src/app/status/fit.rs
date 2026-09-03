//! # `status::fit` — the bar's four named zoom levels
//!
//! Split out of [`super`] under **R2** on 2026-08-24, when O29's Fit height
//! button took that file past 1,500 lines. It joins [`super::zoom`],
//! [`super::page_box`], [`super::filter`] and the rest: `status.rs` owns the
//! bar's **layout and the argument for its order**, and each group owns its
//! own controls.
//!
//! ## ★ The one thing to read before changing anything here
//!
//! The bar's right-hand cluster is laid out **right to left**, so within this
//! group the control added FIRST is drawn RIGHTMOST. The screen reads
//! `Actual size · Fit width · Fit height · Fit page`; the calls below run in
//! the reverse of that. Getting it backwards does not break anything — it
//! silently reorders four controls an operator has learned the positions of.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::status as t;
use crate::viewer::FitMode;

use super::REGION_FIT;

/// `Actual size · Fit width · Fit page`, mirroring View ▸ Zoom under P1a.
///
/// ★ **Two of the three are toggles and one is a button, and that asymmetry
/// is honest rather than sloppy.** `FitMode::Page` and `FitMode::Width` are
/// *modes*: they persist, they re-fit on every window resize, and a control
/// that shows whether you are in one is telling the truth. `FitMode::None`
/// is the absence of a mode, so a "selected" Actual size would light up at
/// any pinned zoom — including 73 % — which is the module docs' ★ defect
/// rendered on screen instead of merely wired. A plain button makes no claim
/// about state.
///
/// Called *last* of the three groups because the layout runs right-to-left;
/// see [`show`].
pub(super) fn group(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let fit = doc.view.fit;
    let rect = ui
        .scope(|ui| {
            // Right-to-left: added first is drawn rightmost, so the screen
            // reads `Actual size · Fit width · Fit height · Fit page`.
            if ui
                .selectable_label(fit == FitMode::Page, t::fit_page())
                .on_hover_text(t::fit_page_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Page));
            }
            // ★ Between Fit page and Fit width on screen — O29. Added
            // second here because the layout is right-to-left, so the bar
            // reads `Actual size · Fit width · Fit height · Fit page` and the
            // two single-axis fits sit beside each other rather than with
            // Fit page wedged between them.
            if ui
                .selectable_label(fit == FitMode::Height, t::fit_height())
                .on_hover_text(t::fit_height_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Height));
            }
            if ui
                .selectable_label(fit == FitMode::Width, t::fit_width())
                .on_hover_text(t::fit_width_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Width));
            }
            // ★ Raises exactly what the ribbon's `view.zoom_actual` raises,
            // including its defect. See the module docs: the fix is a new
            // action variant, not a divergent mirror.
            if ui
                .button(t::fit_actual_size())
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
