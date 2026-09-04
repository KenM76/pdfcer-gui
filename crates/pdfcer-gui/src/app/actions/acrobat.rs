//! # `app::actions::acrobat` — the two halves of handing the document over
//!
//! `OPERATOR_REQUESTS.md` **O122**. Two functions and one ordering.
//!
//! | | Runs when | Does |
//! |---|---|---|
//! | [`PdfcerApp::apply_open_in_acrobat`] | the operator presses the control | asks the right question, and **nothing else** |
//! | [`PdfcerApp::resume_after_open_in_acrobat`] | the operator answered *proceed* | saves if needed, launches, then closes |
//!
//! ## ★★★ The ordering in the second half, which is the whole of the risk
//!
//! **Save → launch → close.** Not save → close → launch, and the difference is
//! everything.
//!
//! `Command::spawn` can fail after every check has passed: the executable was
//! removed since discovery, the operator lacks permission, the process table is
//! full. If the document were closed first, that failure would leave the
//! operator with **no document on screen and no Acrobat**, having pressed one
//! button — a state indistinguishable from the application losing their work,
//! and reached by a code path that had already done everything right.
//!
//! Launching first means a failure leaves the document exactly where it was,
//! open, saved, with a sentence on the bar saying Acrobat would not start and
//! where the path is set. See [`crate::text::acrobat::launch_failed`].
//!
//! ★ The cost of that order is a window in which both programs have the file:
//! between `spawn` returning and `close_document` running, which is one
//! function call and no frame. Acrobat cannot have opened, read and written
//! the file in that interval — and even if it somehow had, pdfcer's close
//! writes nothing. The alternative's failure mode is losing the operator's
//! document off their screen; this one's is theoretical.
//!
//! ## ★★ The save is in-place and its failure stops everything
//!
//! `crate::app::save::save_in_place` is the same verb `file.save` uses, so the
//! bytes written here are the bytes that command writes — and it materialises
//! the whole replacement in a temporary beside the target and renames, so a
//! failure has touched nothing the operator owns.
//!
//! A save that did not happen must **never** be a route to the thing it was
//! supposed to make safe. So a failed save stops the sequence dead: no launch,
//! no close, and a sentence saying so. That is the rule
//! [`crate::dialogs::unsaved`] states about its own resume — *"a save that did
//! not happen must never be a route to discarding the work it was supposed to
//! preserve"* — applied to a sequence where the discarding would be done by
//! another program.

use crate::acrobat;
use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::dialogs::open_in_acrobat::Outcome;
use crate::text::acrobat as t;

impl PdfcerApp {
    /// `Action::OpenInAcrobat` — ask, and do nothing else.
    ///
    /// # Why every refusal here is silent except the one the operator can act
    /// on
    ///
    /// Two of the three ways this can decline to raise a window are states the
    /// ribbon already prevents: no document open (the command carries
    /// `enabled_when("doc.open")`) and no Acrobat (the item carries
    /// `visible_when("acrobat.available")`, so the control is not drawn). They
    /// are handled anyway, because a customized keymap can reach any command
    /// from any state and an arm that assumed otherwise would be a panic
    /// waiting for an operator to find — but they are traced rather than
    /// worded, because there is no surface the operator could have pressed and
    /// therefore nothing to explain.
    ///
    /// The third — a document that has never been saved — **is** worded, in its
    /// own window, because the operator did press a control that was there and
    /// enabled, and a control that does nothing on press is the defect this
    /// project keeps finding.
    pub(super) fn apply_open_in_acrobat(&mut self) {
        let Some(viewer) = self.acrobat.clone() else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "acrobat-declined reason=no-viewer".to_owned()
            });
            return;
        };
        let Status::Open(doc) = &self.status else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "acrobat-declined reason=no-document".to_owned()
            });
            return;
        };

        // ★★★ `save::has_a_file` and `save::has_unsaved_edits` — the two
        // predicates this crate already has, asked here rather than answered
        // again. O65's whole finding was that *"does this document have
        // unsaved edits?"* had grown three different answers in three
        // surfaces; a fourth written here would be the same defect with a new
        // name on it.
        let prompt = acrobat::prompt_for(
            crate::app::save::has_a_file(doc),
            crate::app::save::has_unsaved_edits(doc),
        );
        // Measured from the last SAVE rather than from zero, for
        // `dialogs::unsaved::ask_for`'s reason: *"you have 12 unsaved changes"*
        // after saving eleven of them is a true count of the wrong thing.
        let edits = doc.edit_epoch.saturating_sub(doc.saved_epoch);
        // The file NAME, not the path — see the dialog's `file` field.
        let file = doc.path.file_name().map_or_else(
            || doc.path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );

        self.dialogs
            .ask_open_in_acrobat(prompt, viewer, edits, file);
    }

    /// Drain the Open-in-Acrobat answer and carry it out.
    ///
    /// Called from the frame loop immediately after the dialogs draw, beside
    /// [`PdfcerApp::resume_after_unsaved`] and for that function's reason: it
    /// is not a command but a frame-level observation that a window has been
    /// answered, and the acts it authorises — a write, a close and a process
    /// launch — belong to the application rather than to a dialog.
    ///
    /// See this module's header for why the order is save → launch → close.
    pub(crate) fn resume_after_open_in_acrobat(&mut self) {
        if self.dialogs.take_open_in_acrobat_cancelled() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ Traced, because a cancel and a question nobody answered
                // leave the screen in exactly the same state — the document
                // where it was — and a driven check cannot tell them apart by
                // looking.
                "acrobat-cancelled".to_owned()
            });
            return;
        }
        let Some((Outcome::Proceed, viewer)) = self.dialogs.take_open_in_acrobat_answer() else {
            return;
        };
        let Status::Open(doc) = &self.status else {
            // The document went away between the question and the answer —
            // reachable only by a second window acting in the same frame, and
            // handled rather than assumed away.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "acrobat-abandoned reason=no-document".to_owned()
            });
            return;
        };

        // 1 — SAVE, if there is anything to save. A failure stops everything;
        // see this module's ★★.
        let epoch = doc.edit_epoch;
        if crate::app::save::has_unsaved_edits(doc) && !crate::app::save::save_in_place(doc) {
            // ★ `save_in_place` has already recorded its own failure sentence
            // on the DECLINE row. This one goes on the disclosure row and says
            // what did **not** follow from it — which is the part the operator
            // is waiting to find out, and which the save verb has no way to
            // know because it does not know why it was called.
            //
            // The epoch is the CURRENT one, per `record_note`'s contract: the
            // sentence stands until the next real edit moves past it, which is
            // what retires it without anything having to remember to.
            crate::app::actions::disclosure::record_note(epoch, t::save_failed());
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "acrobat-abandoned reason=save-failed".to_owned()
            });
            return;
        }
        // ★ Re-read after the save. `save_in_place` takes `&self` and the
        // engine's write verb changes nothing about the session, so the path
        // cannot have moved — but reading it once, here, is what makes the
        // launch and the save provably about the same file.
        let path = doc.path.clone();

        // 2 — LAUNCH, before the close. See this module's ★★★.
        if let Err(error) = acrobat::launch(&acrobat::windows::Windows, &viewer, &path) {
            crate::app::actions::disclosure::record_note(
                epoch,
                t::launch_failed(&error.to_string()),
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("acrobat-launch-failed detail={error:?}")
            });
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "acrobat-launched edition={:?} source={:?} file={path:?}",
                viewer.edition, viewer.source
            )
        });

        // ★ The receipt is stamped BEFORE the close, with the epoch that is
        // still current, because after the close there is no document and no
        // epoch to stamp it with. It is the one sentence in this sequence that
        // reports a success, and it is worth having: the document disappearing
        // is unambiguous once you know it was going to, and this is the line
        // that says it did what it said it would.
        crate::app::actions::disclosure::record_note(epoch, t::handed_over(&viewer));

        // 3 — CLOSE. pdfcer gives the file up, which is the operator's own
        // instruction and the reason the window existed.
        self.close_document();
    }
}
