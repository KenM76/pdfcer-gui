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

/// How wide the readout must be so that stepping the zoom never moves the
/// buttons beside it.
///
/// # Why this is measured rather than declared
///
/// Because the thing it has to fit stopped being knowable at authoring time.
/// [`ZOOM_READOUT_WIDTH_PTS`] was written when [`crate::viewer::ZOOM_LADDER`]
/// topped out at 800 % and four characters covered every string the readout
/// could produce. O24 made the ceiling a **preference** whose top preset is
/// `1e12`, and `{:.0}` formatting turns that into `1000000000000%` — fourteen
/// characters. A constant cannot cover a range the operator chooses.
///
/// So the reserve is the galley width of the widest string this ceiling can
/// ask for, floored at the old constant.
///
/// # ★★ Why this is not the feedback loop that has bitten this project before
///
/// R128 and the fit-zoom defect were both *a measurement of laid-out content
/// fed back into the size of the thing that lays it out*, which oscillates.
/// This measures a **string that does not depend on the width** —
/// `zoom_percent(ceiling)` is the same text whatever the reserve turns out to
/// be — so there is no loop to close. The output is a pure function of
/// (ceiling, font), and both are stable across a frame.
///
/// The width changes only when the operator picks a different ceiling, which
/// is an explicit act in a popup, not something that happens under the pointer
/// while stepping.
///
/// ★ `+ 2.0`: `Button::frame(false)` still lays out with the style's button
/// padding, and a galley measured to the pixel against a rect measured to the
/// pixel truncates on the last glyph under rounding. Two points is the
/// smallest allowance that is visibly never wrong, and it is stated here
/// rather than folded into the floor so that the floor keeps meaning
/// "the old reserve".
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

    /// ★★★ The premise the old constant asserted, measured — and it is false.
    ///
    /// `ZOOM_READOUT_WIDTH_PTS`' doc comment said 46 pt was *"wide enough for
    /// four characters, which is the whole range `ZOOM_LADDER` can produce."*
    /// This is that sentence turned into an assertion, and it fails: O24 made
    /// the ceiling a preference topping out at `MAX_MAX_ZOOM_PERCENT`, and
    /// `{:.0}` renders that as fourteen characters.
    ///
    /// A test on the STRING rather than on the pixel width, deliberately: the
    /// defect is not "46 pt is the wrong number", it is "the readout's content
    /// outgrew what the number was chosen for". A width assertion would pin a
    /// font metric and would have to be re-tuned whenever the face changed; the
    /// character count is the durable statement.
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
