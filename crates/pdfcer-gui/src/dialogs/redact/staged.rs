//! # `dialogs::redact::staged` — the phase for a document whose removal is
//! already armed
//!
//! **New 2026-09-05, with `pdfcer-core` `Pass 250.2`.** It draws one heading,
//! one paragraph and one control, and it exists as a module of its own for a
//! seam rather than for a line count — [`super`] was at 1,318 lines before this
//! work and R2's ceiling was not in sight.
//!
//! ## ★ The seam, argued
//!
//! [`super`] answers *"what will applying do, and may the operator commit
//! it?"*. Every part of it — the measured report, the destination choice, the
//! three acknowledgements, the confirm control read one frame late — is
//! machinery for taking a decision that has not been taken yet.
//!
//! This file answers a different question, asked by a different person: *"I
//! already decided. What did I decide, and can I change my mind?"* It has no
//! report (the numbers are stale by the time it is drawn — see below), no
//! acknowledgements (there is nothing to consent to), no destination (the
//! destination is whichever save verb he reaches for) and no confirm control.
//! Sharing a file with the transaction would have meant a `match` arm inside
//! `report`, inside `gates`, and inside `commit`, each testing the same phase —
//! which is the shape [`super`]'s own §3 warns about in a different context: a
//! control whose meaning depends on state somewhere else.
//!
//! ## ★★★ Why it quotes no numbers, which is the decision worth reading
//!
//! The obvious body for this phase is the report the operator agreed to: *"4
//! regions across 2 pages will be removed"*. It is not drawn, and the reason is
//! `crate::redact::StagedRedaction::report`'s own:
//!
//! > A **preview** and not a receipt … the actual removal re-runs at save over
//! > the then-current state, so if the operator edits in between, the saved
//! > result reflects the edits.
//!
//! Between the arming and this frame he may have marked two more regions, taken
//! one off, undone six steps, or edited the text under a mark. Every one of
//! those changes what a save would remove, and none of them is visible in a
//! number captured when he pressed the button. **A stale measurement presented
//! as a current one is worse than no measurement**, on the one surface where
//! the operator's whole reason for reading is to check what is about to
//! happen — and this shell has a standing rule about exactly that shape
//! (`crate::dialogs::redact` §2: the report has to be a measurement rather than
//! a prediction, which is why the transaction runs the removal on open).
//!
//! The honest alternative — re-running the removal here to get fresh numbers —
//! is a full rewrite of the document every time the window is opened, for a
//! read-only glance. It is available (`EditSession::save_applying_redaction`
//! takes `&self`) and it is not taken, because the fresh numbers would answer a
//! question nobody is asking at this point: he is here to call it off or to go
//! away, and both answers are the same whatever the count is.
//!
//! ## What it does NOT draw, and it is a rule-4 statement
//!
//! No badge, no tint, no dashed outline, no progress. A staged removal is
//! invisible on the canvas by design (`crate::redact` §1.0.3) and this window is
//! the off-canvas place where it is disclosed instead. The status line and the
//! tab's unsaved marker are the other two.

use egui_shell::theme::Theme;

use crate::text::redact as t;

/// The control that un-stages a removal, published so `tools/ui-verify` can
/// click it.
///
/// ★ Declared **only while it is on screen**, so its absence from a trace is
/// evidence that nothing is armed on this document rather than evidence that
/// the build has lost the control. The same asymmetry
/// `super::REGION_DESTINATION_REPLACE` carries.
const REGION_CANCEL: &str = "redact-apply-cancel-staged"; // ui-text-exempt: trace region name, never displayed

/// **Draw the staged phase. Returns `true` when the operator asked to call the
/// removal off.**
///
/// A `bool` out rather than an `&mut` flag in, so the whole body is a pure
/// function of the theme and the caller owns every piece of mutable state —
/// `crate::viewer`'s standing split, applied to the one control in this window
/// that changes what the next `Ctrl+S` does.
///
/// # ★ The order: heading, then the paragraph, then the control
///
/// The control is last and it is the only thing on screen that acts, so there
/// is no gate on it and none is wanted. Calling a removal off **loses nothing**
/// — the marks stay, the content stays, and it can be armed again in two clicks
/// — which is the opposite of every other control in this window, and a
/// checkbox in front of it would teach the operator that this dialog's
/// acknowledgements are ceremony rather than consequence.
pub(super) fn body(ui: &mut egui::Ui, theme: &Theme) -> bool {
    ui.label(t::staged_heading());
    ui.add_space(6.0);
    // ★ `danger`, matching the staging disclosure the transaction draws above
    // its confirm control, and for the same reason: an armed removal is not a
    // notice. The palette's split is that `notice` means *"worth knowing and
    // nothing is broken"*, and a document that cannot be saved by any ordinary
    // means until this is resolved is not that.
    ui.label(egui::RichText::new(t::staged_body()).color(theme.palette.danger));
    ui.add_space(10.0);
    let cancel = ui.button(t::cancel_button_staged());
    crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
    let clicked = cancel.clicked();
    // ★★ `.rect` and `.clicked()` are read BEFORE the hover text, because
    // `on_hover_text` consumes the response — the borrow order copied from
    // `dialogs::formfield`, recorded here so a reordering does not silently
    // stop publishing the region.
    cancel.on_hover_text(t::cancel_button_staged_tooltip());
    clicked
}
