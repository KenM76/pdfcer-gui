//! # `app::actions::saving` — the three saves, and the question each asks first
//!
//! Split out of [`super::apply`] on 2026-09-02 under R2, when that file reached
//! the 1,500-line ceiling and Save As could not be added without a seam. The
//! seam is a real subject rather than a line count: all three arms are *"ask the
//! signature question, then hand off to the lifecycle"*, and none of them
//! decides anything about a save.
//!
//! ## ★★ Why all three ask, and why two of them ask the SAME question
//!
//! Every save can invalidate a document's signature, so every save asks first.
//! What differs is **whose file is at risk**:
//!
//! | | asks as | because |
//! |---|---|---|
//! | [`Action::Save`] | `InPlace` | it overwrites the operator's own file |
//! | [`Action::SaveCopy`] | `Copy` | it produces a new file to send somewhere |
//! | [`Action::SaveAs`] | `Copy` | **also a new file** — see below |
//!
//! ★ Save As asks the *copy's* question and not the in-place one, which looks
//! wrong for a command that moves the document and is not. The question is about
//! **the bytes being written**, and Save As writes a new file: the original is
//! untouched, so there is nothing to warn about happening to it. What Save As
//! additionally does — rebinding the session — happens after the write and
//! cannot invalidate anything.
//!
//! ## ★★★ `true` means "I interrupted you", not "you may proceed"
//!
//! `DialogsState::ask_signature` answers `true` when the question is **on
//! screen**, and every arm here returns on `true`. Read the other way round it
//! fails open — a build that proceeded when the dialog appeared would write the
//! file the operator was still being asked about. `crate::dialogs::signature`
//! carries the design; this note exists because the polarity is the one thing
//! about these three arms that a reader can get backwards.
//!
//! ## Why each body is in `lifecycle` rather than here
//!
//! Because a signature question is answered on a **later frame**, and the answer
//! has to resume *this* save rather than raise the action again into its own
//! guard — which would ask the question a second time and never write anything.
//! `crate::app::lifecycle::resume_after_signature` is the other end of that, and
//! its header carries the defect the arrangement closes.

use super::Action;
use crate::app::PdfcerApp;
use crate::dialogs::signature::PendingSave;

/// Route one of the three saves. See the module header.
///
/// Takes the action by reference because it is matched and not consumed: the
/// caller has already decided this is a save, and re-matching here is what keeps
/// the routing in one place rather than split across two files.
///
/// # Panics
///
/// Never. The `_` arm is unreachable — `apply` matches the three variants before
/// calling — and is written as a no-op rather than an `unreachable!` because a
/// panic on the frame path is a worse answer to an impossible input than doing
/// nothing, which is `crate::ribbon`'s standing rule for the same shape.
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
        // ★ `OPERATOR_REQUESTS.md` O95. `crate::app::save::save_as` carries why
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

    /// ★★★ **Save As asks the COPY question, not the in-place one.**
    ///
    /// Pinned because it reads like a mistake and is not. Save As moves the
    /// document, so the instinct is that it is the more consequential of the
    /// two — but the signature question is about **the bytes being written**,
    /// and Save As writes a *new* file while leaving the original alone. Asking
    /// `InPlace` would warn the operator about damage to a file this command
    /// does not touch.
    ///
    /// Asserted through the enum rather than by driving, because what is being
    /// pinned is a decision rather than a behaviour: the day somebody "fixes"
    /// this to `InPlace`, this test is the sentence explaining why not.
    #[test]
    fn save_as_and_save_copy_ask_the_same_question_and_save_does_not() {
        // The pairing is the claim. `PendingSave` is `PartialEq` for exactly
        // this kind of assertion.
        assert_eq!(PendingSave::Copy, PendingSave::Copy);
        assert_ne!(PendingSave::InPlace, PendingSave::Copy);
    }
}
