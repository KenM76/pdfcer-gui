//! # `dialogs::redact::staged` — the phase for a document whose removal is
//! already armed
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/redact/staged.md`.

use egui_shell::theme::Theme;

use crate::text::redact as t;

/// The control that un-stages a removal, published so `tools/ui-verify` can
/// click it.
///
/// Declared **only while it is on screen**, so its absence from a trace is
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
/// # The order: heading, then the paragraph, then the control
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
    // `danger`, matching the staging disclosure the transaction draws above
    // its confirm control, and for the same reason: an armed removal is not a
    // notice. The palette's split is that `notice` means *"worth knowing and
    // nothing is broken"*, and a document that cannot be saved by any ordinary
    // means until this is resolved is not that.
    ui.label(egui::RichText::new(t::staged_body()).color(theme.palette.danger));
    ui.add_space(10.0);
    let cancel = ui.button(t::cancel_button_staged());
    crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
    let clicked = cancel.clicked();
    // `.rect` and `.clicked()` are read BEFORE the hover text, because
    // `on_hover_text` consumes the response — the borrow order copied from
    // `dialogs::formfield`, recorded here so a reordering does not silently
    // stop publishing the region.
    cancel.on_hover_text(t::cancel_button_staged_tooltip());
    clicked
}
