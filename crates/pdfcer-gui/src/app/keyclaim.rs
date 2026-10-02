//! # `app::keyclaim` — Tab taken from egui's focus walk before the frame reads it
//!
//! An egui plugin registered after the scripted pointer, so a driven Tab and an
//! OS Tab take the same route. A form-field or object ring takes Tab through
//! [`crate::canvas::tabnav::claim`]; an open text draft takes it through
//! [`crate::canvas::textedit::claim_tab`]. Either way egui never moves keyboard
//! focus on it, so a later Enter cannot press whatever button focus landed on.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/keyclaim.md`.

/// The plugin; [`super::configure_context`] adds it after the scripted pointer.
pub struct KeyClaim;

impl egui::Plugin for KeyClaim {
    fn debug_name(&self) -> &'static str {
        "pdfcer-key-claim"
    }

    fn input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        crate::canvas::tabnav::claim(ctx, input);
        crate::canvas::textedit::claim_tab(ctx, input);
    }
}
