//! # `app::keyclaim` — Tab taken from egui's focus walk before the frame reads it
//!
//! An egui plugin registered after the scripted pointer, so a driven Tab and an
//! OS Tab take the same route. A form-field or object ring takes Tab through
//! [`crate::canvas::tabnav::claim`]; an open text draft takes it through
//! [`crate::canvas::textedit::claim_tab`]. Either way egui never moves keyboard
//! focus on it, so a later Enter cannot press whatever button focus landed on.
//! An open draft also keeps the arrow keys
//! ([`crate::canvas::textedit::hold_arrows`]). Every change of root keyboard
//! focus is traced as `keyboard-focus`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/keyclaim.md`.

/// The plugin; [`super::configure_context`] adds it after the scripted pointer.
#[derive(Default)]
pub struct KeyClaim {
    /// Root keyboard focus at the end of the last pass.
    focus: Option<egui::Id>,
}

impl egui::Plugin for KeyClaim {
    fn debug_name(&self) -> &'static str {
        "pdfcer-key-claim"
    }

    fn input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        crate::canvas::tabnav::claim(ctx, input);
        crate::canvas::textedit::claim_tab(ctx, input);
        crate::canvas::textedit::hold_arrows(ctx, input);
    }

    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx();
        if ctx.viewport_id() != egui::ViewportId::ROOT {
            return;
        }
        let now = ctx.memory(egui::Memory::focused);
        if now == self.focus {
            return;
        }
        self.focus = now;
        let draft = crate::canvas::textedit::read(ctx).is_some();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "keyboard-focus to={} draft={draft}",
                now.map_or_else(|| "none".to_owned(), |id| format!("{:x}", id.value()))
            )
        });
    }
}
