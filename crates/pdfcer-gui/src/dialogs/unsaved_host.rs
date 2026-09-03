//! # `dialogs::unsaved_host` — raising and draining the unsaved question
//!
//! The four accessors [`super::DialogsState`] offers for
//! [`crate::dialogs::unsaved`], split out of [`super`] on 2026-09-02 under R2
//! when that file crossed the 1,500-line ceiling.
//!
//! ## ★★ The seam is a real one: a question with THREE answers
//!
//! Every other dialog this state hosts parks one thing — an outcome — and the
//! host drains it. This one has three states and only two of them are an
//! outcome:
//!
//! | the operator did | parked where | drained by |
//! |---|---|---|
//! | Save, Save a copy, Save all, Discard | on the dialog | [`super::DialogsState::take_unsaved_answer`] |
//! | **Cancel** | on the **host**, at the drop site | [`super::DialogsState::unsaved_cancelled`] |
//! | nothing yet | nowhere | — |
//!
//! ★★★ The middle row is the one that cost a defect. A Cancel parks no outcome,
//! so the retire rule drops the window on it and the answer went with it. That
//! was invisible for as long as every caller read a Cancel as *"nothing
//! happened"* — true for a tab close, where the tab simply stays — and became a
//! defect the moment `OPERATOR_REQUESTS.md` O102's quit cycle needed to tell
//! *"they cancelled"* from *"they have not answered yet"*. Without the
//! distinction it re-asked on the very next frame, forever.
//!
//! ★ Found by driving, not by reading: the check saw two `unsaved-asked` lines
//! and no `quit-cancelled`.

use crate::app::state::Status;

use super::{DialogsState, unsaved};

impl DialogsState {
    /// **Ask about `intent` if the open document has unsaved edits.**
    ///
    /// Returns `true` when the question was raised and the caller must
    /// **stop** — the intent is now this window's to resume. `false` means
    /// there was nothing to ask about and the caller proceeds unchanged.
    ///
    /// # ★ Why the return value is "did I interrupt you" rather than "may I
    /// proceed"
    ///
    /// Both spellings work and only one of them is safe to get wrong. A guard
    /// read as *"may I proceed"* fails **open** when somebody inverts it or
    /// forgets it: the document is destroyed. This one fails **closed** — a
    /// missing `if` means the question is asked and its answer resumes the
    /// intent anyway, so the operator sees one redundant prompt rather than
    /// losing their afternoon.
    ///
    /// The already-open guard matters more here than on any other dialog in
    /// this struct: without it, a keymap chord repeated while the question is
    /// on screen would replace the pending intent with a second one, and the
    /// operator would answer a question about Close and get an Open.
    pub fn ask_unsaved(&mut self, status: &Status, intent: unsaved::PendingIntent) -> bool {
        if self.unsaved.is_some() {
            // Already asking. Swallow the second request rather than stacking
            // it: the operator is looking at a question and has not answered
            // it, and the honest reading of a second press is impatience.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "unsaved-ask-ignored reason=already-asking".to_owned()
            });
            return true;
        }
        let Some(dialog) = unsaved::ask_for(status, intent) else {
            return false;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "unsaved-asked".to_owned()
        });
        self.unsaved = Some(dialog);
        true
    }

    /// [`Self::ask_unsaved`], told how many documents are dirty — the quit
    /// cycle's entry point (`OPERATOR_REQUESTS.md` O102).
    ///
    /// ★ The count decides only whether *Save all* is drawn.
    pub fn ask_unsaved_in_cycle(
        &mut self,
        status: &Status,
        intent: unsaved::PendingIntent,
        dirty: usize,
    ) -> bool {
        if self.unsaved.is_some() {
            return true;
        }
        let Some(dialog) = unsaved::ask_for_cycle(status, intent, dirty) else {
            return false;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("unsaved-asked cycle=true dirty={dirty}")
        });
        self.unsaved = Some(dialog);
        true
    }

    /// **Was the unsaved question answered with Cancel?**
    ///
    /// ★ Drained by the quit cycle, which needs to know that the operator said
    /// no — a Cancel closes the window and parks no outcome, so
    /// `take_unsaved_answer` reports nothing at all and the cycle would
    /// otherwise re-ask on the very next frame, forever.
    pub fn unsaved_cancelled(&mut self) -> bool {
        // ★ Drains the PARKED flag; the dialog is already gone.
        std::mem::take(&mut self.unsaved_cancelled)
    }
}
