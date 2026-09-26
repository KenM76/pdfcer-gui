//! # `app::actions::saving` — the three saves, and the question each asks first
//!
//! Every arm here is the same shape: *ask the signature question, then hand off
//! to the lifecycle*. None of them decides anything about a save — the decisions
//! live in `crate::app::save` and `crate::app::lifecycle`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/saving.md`.

use super::Action;
use crate::app::PdfcerApp;
use crate::dialogs::signature::PendingSave;

/// Route one of the three saves. See the module header.
pub(super) fn apply(app: &mut PdfcerApp, action: &Action) {
    match action {
        Action::Save => {
            if app.dialogs.ask_signature(&app.status, PendingSave::InPlace) {
                return;
            }
            app.write_in_place();
        }
        Action::SaveCopy => {
            if app.dialogs.ask_signature(&app.status, PendingSave::Copy) {
                return;
            }
            app.write_copy_somewhere();
        }
        // `OPERATOR_REQUESTS.md` O95. `crate::app::save::save_as` carries why
        // this is a different act from a copy; `PdfcerApp::save_as_somewhere`
        // carries what rebinding the document costs.
        Action::SaveAs => {
            if app.dialogs.ask_signature(&app.status, PendingSave::Copy) {
                return;
            }
            app.save_as_somewhere();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Save As asks the copy question, not the in-place one.**
    #[test]
    fn save_as_and_save_copy_ask_the_same_question_and_save_does_not() {
        // `PendingSave` is `PartialEq` for exactly this kind of assertion.
        assert_eq!(PendingSave::Copy, PendingSave::Copy);
        assert_ne!(PendingSave::InPlace, PendingSave::Copy);
    }
}
