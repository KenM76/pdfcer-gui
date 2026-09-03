//! # `app::actions::document` — the actions that decide WHICH document is on
//! screen, and the two guards the destructive ones share
//!
//! ## Why this is a file of its own
//!
//! `apply.rs` crossed R2's 1,500-line gate on 2026-08-19 when the unsaved-edits
//! guard landed, and `tools/gates/check-file-size.sh`'s own header says what
//! not to do about that: *"Split the module along its seams — one subject per
//! file — rather than raising the limit."*
//!
//! This is the seam, and it was already drawn in prose before it was drawn in
//! files. `apply`'s match has a block at the top whose own comment reads: *"The
//! three actions that are about WHICH document is open, matched BEFORE the
//! guard below."* Everything below that block acts **on** the open document;
//! everything in it decides **which** document is open, or whether there is
//! one. That is a different subject with a different failure mode — the arms
//! below can be wrong about a page, and these can be wrong about an afternoon's
//! work.
//!
//! ## ★★ 2026-08-19: three of the five arms stopped needing the guards, and
//! that is the point of the change rather than a relaxation of it
//!
//! There are now **five** arms here, and they split two-three:
//!
//! | arm | discards a document? | guards |
//! |---|---|---|
//! | `apply_open` | **no** — it adds a tab | none |
//! | `apply_new` | **no** | none |
//! | `apply_new_sized` | **no** | none |
//! | `apply_close` | yes — the one on screen | both |
//! | `apply_close_document` | yes — the one whose tab was clicked | both |
//!
//! Before the document tabs, Open and New *replaced* what was open, and the
//! guard on them was the most valuable one in the file — see `apply_open`,
//! which keeps the old argument verbatim because it was right. Now they park
//! the open document and add a tab, so the question they used to ask —
//! *"your unsaved edits will be lost"* — would be **false**, and a
//! confirmation that says something untrue is how an operator learns to
//! dismiss confirmations unread.
//!
//! So the protection did not weaken; it moved to where the loss now happens.
//! And the loss now happens in **two** places rather than one, which is
//! exactly what
//! [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
//! is shaped to notice.
//!
//! ## ★ The two guards, in order, and why the order is not interchangeable
//!
//! Both closing arms ask the same two questions in the same sequence:
//!
//! | # | question | answer | why first / second |
//! |---|---|---|---|
//! | 1 | **Is a save in flight?** (`PdfcerApp::save_pending`) | decline outright, trace it | the document's bytes are mid-write; there is no answer the operator could give that would make proceeding safe |
//! | 2 | **Are there unsaved edits?** (`DialogsState::ask_unsaved`) | **ask**, and resume afterwards | there is an answer the operator can give, and it is theirs to give |
//!
//! Reversing them would put a question in front of an operator whose answer
//! cannot be honoured — they would press *Close without saving* and be declined
//! anyway, which reads as a broken button.
//!
//! **They are two predicates, not one, and conflating them is the mistake this
//! file exists to prevent.** `crate::app::lifecycle::save_pending` carries the
//! whole argument: it asks *"is a save in flight"*, is permanently `false`
//! because `file.save_copy` is synchronous, and is **not** *"are there unsaved
//! edits?"* — a successful save-a-copy leaves the document exactly as unsaved
//! as it was, because the copy went somewhere else. `dialogs::ocr`'s
//! `UnsavedEdits` refusal reads `edit_epoch != 0` and would break the moment
//! somebody merged them.
//!
//! ## ★★ The defect this file's shape closes
//!
//! Guard 2 did not exist until 2026-08-19. Every one of the four arms that
//! then existed destroyed every edit made since the file was opened, silently,
//! with no prompt and no undo — while `file.close`'s shipped tooltip promised
//! the operator *"You are asked what to do about unsaved edits first."*
//!
//! It was found by an audit against `pdfcer`'s capability register, not by a
//! test and not by use, and the reason it survived so long is worth keeping:
//! **the guard that should have caught it existed, was well argued, was
//! correct, and was answering a different question.** A reader arriving at
//! `Action::Close` saw a guard, saw a doc comment explaining the guard, and had
//! no reason to ask whether it was the guard the tooltip was describing.
//!
//! Putting every such arm in one file with the guard table above is the
//! structural half of not repeating that. The other half is
//! [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`],
//! which fails when a sixth one arrives without the guards — and which had to
//! be rewritten when the count moved, because its first form counted *arms*
//! and the property was never about the count.

use crate::app::PdfcerApp;
use crate::dialogs::unsaved::PendingIntent;

impl PdfcerApp {
    /// `Action::Open` — open the document at `path`, **in a tab of its own**.
    ///
    /// With nothing open this is the **ordinary** case: it is how an operator
    /// gets their first document after launching with no argument. That is why
    /// this arm is matched before `apply`'s document guard rather than being
    /// subject to it.
    ///
    /// ★★ **This arm used to ask about unsaved edits and no longer does, and
    /// the change is the whole point of the multi-document work.**
    ///
    /// What it used to say, kept because the reasoning was correct for the
    /// application it was written about:
    ///
    /// > ★ **The arm that needed the unsaved-edits question most**, and it is
    /// > worth saying why it is not Close. An operator who has marked up a
    /// > drawing and then opens the next one has destroyed exactly as much
    /// > work as one who pressed Close — and is far more likely to do it,
    /// > because opening the next file is what you do all day, whereas closing
    /// > a document deliberately is something you do at the end of one.
    ///
    /// Every word of that was true while an Open **replaced** the document.
    /// Since 2026-08-19 it does not: `open_path` parks what was open and adds
    /// a tab. Nothing is discarded, so there is nothing to ask about — and
    /// asking anyway would be worse than useless, because the question
    /// *"Open another document? Your unsaved edits will be lost."* would be
    /// **false**.
    ///
    /// The protection the old guard gave is not lost. It moved to where the
    /// loss now actually happens, which is a close — and there are two of
    /// those.
    ///
    /// The `save_pending` guard went with it, for the same reason: it means
    /// *this document's bytes are mid-write*, and opening a different document
    /// does not touch them.
    pub(super) fn apply_open(&mut self, path: std::path::PathBuf) {
        self.open_path(path);
    }

    /// `Action::OpenWithPassword` — the retry an encrypted document needs.
    ///
    /// `OPERATOR_REQUESTS.md` O108. Raised only by
    /// [`crate::dialogs::password::PasswordDialog`], which is the only surface
    /// that can obtain a password.
    ///
    /// ★ The password is **borrowed**, never cloned into a second place. It
    /// arrives inside the action, is handed to `Document::load_with_password`
    /// through `Secret::expose`, and the action is dropped with it. There is no
    /// step here that stores it.
    pub(super) fn apply_open_with_password(
        &mut self,
        path: std::path::PathBuf,
        password: &crate::secret::Secret,
    ) {
        match self.open_path_with_password(path, password) {
            // ★ The prompt stays up and says WHICH failure it was. The two are
            // different instructions to the operator — "try again" against
            // "pdfcer cannot open this file however correct your password is" —
            // and the engine separated them precisely so this last step could.
            Some(why) => {
                self.dialogs.reject_password(why);
            }
            // Opened. Close the prompt; the document is on screen behind it.
            None => self.dialogs.password_accepted(),
        }
    }

    /// `Action::New` — a blank document, in a tab of its own.
    ///
    /// Unguarded since 2026-08-19, for [`Self::apply_open`]'s reason: it adds
    /// a document rather than replacing one.
    pub(super) fn apply_new(&mut self) {
        self.new_document();
    }

    /// `Action::NewSized` — the same, with a page box the operator chose.
    ///
    /// ★ Beside the plain New, and unguarded with it since 2026-08-19. The two
    /// used to be *"the same arm shape, deliberately next to its twin, so that
    /// a change to what either guard means cannot be applied to one New and
    /// missed on the other"* — and the guards are now gone from both, which is
    /// that same property arrived at by subtraction.
    pub(super) fn apply_new_sized(&mut self, width_pt: f64, height_pt: f64) {
        // The lower-left corner is the origin: a new page has nothing to offset
        // from, and `Action::NewSized`'s own docs say why the action carries a
        // size rather than a rectangle.
        self.new_document_sized(pdfcer_core::page_tree::Rect::from_corners(
            0.0, 0.0, width_pt, height_pt,
        ));
    }

    /// `Action::Close` — put the document away.
    ///
    /// With nothing open this is a no-op that must still not be reached through
    /// a path that assumes a document, which is the other half of why this
    /// family is matched before `apply`'s guard.
    pub(super) fn apply_close(&mut self) {
        // A close the operator started themselves ends any `Close others`
        // sequence that was waiting for an answer. See
        // `PdfcerApp::closing_others`.
        self.closing_others = None;
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close-declined reason=save-pending".to_owned()
            });
            return;
        }
        if self.dialogs.ask_unsaved(&self.status, PendingIntent::Close) {
            return;
        }
        self.close_document();
    }

    /// `Action::CloseDocument` — close the tab at `slot`, which may not be the
    /// one on screen.
    ///
    /// The fifth arm, and it asks the same two questions in the same order as
    /// [`Self::apply_close`]. What it adds is one step between them, and that
    /// step is the reason it is a separate function rather than a parameter:
    ///
    /// > A **modified** background tab is brought to the front *before* the
    /// > question is asked.
    ///
    /// Because the question is *"you have unsaved edits — save a copy, close
    /// without saving, or cancel?"*, and an operator being asked that about a
    /// document they cannot see has no way to decide: they would be answering
    /// about whatever is on screen. Word and VS Code both switch to the tab
    /// they are about to prompt over, and this does the same.
    ///
    /// A **clean** background tab closes where it stands. Switching to it
    /// first would be a visible jolt — the canvas re-rendering another
    /// document for one frame — in service of a question that is not going to
    /// be asked.
    ///
    /// That ordering is also what makes the resume correct without a new
    /// [`PendingIntent`]: by the time the dialog is up, the document being
    /// asked about *is* the active one, so `PendingIntent::Close` resuming
    /// through `close_document` closes exactly what the operator was looking
    /// at. A `PendingIntent::CloseSlot(n)` would carry a slot number across a
    /// dialog, and slots renumber when a tab closes.
    pub(super) fn apply_close_document(&mut self, slot: usize) {
        // As `apply_close`. `apply_close_other_documents` re-parks it AFTER
        // this returns, so its own loop is unaffected.
        self.closing_others = None;
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("close-document-declined slot={slot} reason=save-pending")
            });
            return;
        }
        // ★★★ O65: `save::has_unsaved_edits`, not `session.is_modified()`.
        // The engine's answer is "differs from the BASE revision", which an
        // incremental save cannot clear — so a **saved** background tab was
        // treated as modified, which yanked the canvas to it (`activate_slot`
        // below) before asking a question that should not have been asked.
        let modified = matches!(
            self.slot(slot),
            Some(crate::app::state::Status::Open(doc))
                if crate::app::save::has_unsaved_edits(doc)
        );
        if !modified {
            self.close_slot(slot);
            return;
        }
        self.activate_slot(slot);
        if self.dialogs.ask_unsaved(&self.status, PendingIntent::Close) {
            return;
        }
        self.close_document();
    }

    /// `Action::CloseOtherDocuments` — close everything except the tab at
    /// `keep`.
    ///
    /// ★★ **The sixth arm, and it has both guards by DELEGATION** rather than
    /// by carrying its own copies. Every close it performs goes through
    /// [`Self::apply_close_document`], which is where the guards live, so there
    /// is no second place for *"does this ask about unsaved edits?"* to be
    /// answered differently.
    ///
    /// This is also why [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
    /// still holds with a sixth arm present: this body names a close verb only
    /// through its sibling, and the sibling is checked.
    ///
    /// # ★ It closes from the RIGHT, and `keep` is adjusted as it goes
    ///
    /// Slots renumber every time one is removed, so the loop takes the
    /// **rightmost tab that is not `keep`** each pass — which is either the
    /// last one or, when the last one is the keeper, the one before it. Only
    /// when the victim was *below* `keep` does anything shift, and then by
    /// exactly one, which is the whole of the bookkeeping.
    ///
    /// The alternative — a `for` over a snapshot of the indices — is the
    /// obvious version and is wrong after the first close, because every index
    /// it holds names a different document from then on.
    ///
    /// # ★★ It survives the unsaved-edits question, which is what makes it
    /// usable
    ///
    /// A modified document brings itself to the front and asks, and answering
    /// takes a frame — so the loop cannot simply continue. It parks `keep` in
    /// [`PdfcerApp::closing_others`] and returns; [`PdfcerApp::resume_after_unsaved`]
    /// picks it up once the operator has answered and runs the rest.
    ///
    /// Without that, *"close others"* over four marked-up drawings would close
    /// one per press, and an operator would reasonably conclude the command was
    /// broken.
    ///
    /// # A cancelled answer stops the whole sequence
    ///
    /// Cancelling produces **no answer**, so `resume_after_unsaved` never runs
    /// and the parked `keep` is never picked up — the sequence simply ends.
    /// That is deliberate rather than incidental: *"close the others"* is a
    /// convenience, and somebody who cancels halfway has said something about
    /// the gesture, not about one document.
    ///
    /// `pub(crate)` rather than `pub(super)`, and the one arm in this file that
    /// is: `crate::app::lifecycle::resume_after_unsaved` continues the sequence
    /// once an answer arrives, and it lives in `app` rather than in
    /// `app::actions`. Routing the resume back through an `Action` was the
    /// alternative and would have re-entered the guard that has just been
    /// answered — the same loop `resume_after_unsaved`'s own header describes
    /// and refuses.
    pub(crate) fn apply_close_other_documents(&mut self, keep: usize) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close-others keep={keep} of={}",
                self.document_count()
            )
        });
        let mut keep = keep;
        loop {
            let count = self.document_count();
            if count <= 1 || keep >= count {
                break;
            }
            // The rightmost that is not the keeper. `count >= 2` here, so when
            // the last tab IS the keeper there is always one before it.
            let victim = if count - 1 == keep {
                count - 2
            } else {
                count - 1
            };
            self.apply_close_document(victim);
            if self.document_count() == count {
                // Nothing went. Either a dialog is now up — the ordinary case
                // for a modified document — or a guard declined. Park the
                // keeper and let the resume continue; a cancel simply never
                // resumes.
                self.closing_others = Some(keep);
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "close-others-paused at={victim} left={count} keep={keep}"
                    )
                });
                return;
            }
            if victim < keep {
                keep -= 1;
            }
        }
        self.closing_others = None;
    }
}

#[cfg(test)]
mod tests {
    /// ★★ **Every action here that DISCARDS a document asks about unsaved
    /// edits, in the right order.**
    ///
    /// The gate that would have caught the 2026-08-19 defect, and the shape it
    /// had to be rewritten into on the same day.
    ///
    /// # ★★ Why it was rewritten, which is the more useful half
    ///
    /// Its first form asserted *"there are exactly four arms in this file and
    /// all four call `ask_unsaved`"*. The count was load-bearing — it was the
    /// floor that stopped a rename making the loop iterate zero times and
    /// report success.
    ///
    /// Then the document tabs landed and **three of the arms stopped
    /// discarding anything**, while a fifth arrived that does. The old test
    /// failed, correctly, and the tempting repair was to move `4` to `5`. That
    /// repair would have been wrong in the direction this project keeps
    /// finding: it would have demanded a guard on `apply_open`, whose guard is
    /// now a *false statement to the operator*, and the test would have
    /// enforced a lie.
    ///
    /// The property was never "how many arms are there". It is **an arm that
    /// can destroy a document must ask first**. So that is what is asserted:
    /// any body naming a close verb must also name both guards, in order. The
    /// counts remain as floors — an instrument that cannot fail detects
    /// nothing — but they are floors on *both* populations now, so neither
    /// "no arms were found" nor "no destructive arms were found" can pass
    /// silently.
    ///
    /// # Why it reads the source rather than driving the functions
    ///
    /// Driving them is not possible in a unit test and the reason is the point:
    /// three of the five end in `open_path` / `new_document` /
    /// `new_document_sized`, which build real `EditSession`s, and the two that
    /// do not would pass trivially by having no document to ask about. A
    /// behavioural test here would exercise the **absence** of the guard's
    /// precondition rather than the presence of the guard.
    ///
    /// Crude, and deliberately so — the same trade this project made for the
    /// settings-coverage gate. A crude check that fails when the guard is
    /// dropped beats an exact one that cannot run.
    #[test]
    fn every_action_that_discards_a_document_asks_about_unsaved_edits() {
        const SRC: &str = include_str!("document.rs");
        // The function bodies, split on their own signatures. `skip(1)` drops
        // everything before the first, which is the module header.
        //
        // ★★ The marker is ASSEMBLED from two pieces rather than written as one
        // literal, and this test's first two drafts are why.
        //
        // The scan looks for the function signatures. Writing that signature
        // out as a single string — here, or in a comment explaining why not to
        // — puts an extra copy of it into the very file being scanned, and the
        // split finds one body too many. Both drafts did it: the first in the
        // `split` call, the second in the comment warning about the first.
        //
        // Funny, and the shape is not. **The instrument was counting itself**,
        // and the spurious body would have contained `ask_unsaved` and
        // `save_pending` — they appear in the assertion messages — so it would
        // have passed every check below. `CONTINUE.md` §7's rule arriving from
        // a direction nobody predicted: a source-scanning test is part of its
        // own corpus, and the floor assertion is what noticed.
        let marker = format!("    pub(super) {}", "fn apply_");
        let bodies: Vec<&str> = SRC.split(marker.as_str()).skip(1).collect();
        assert!(
            bodies.len() >= 5,
            "found {} arms; the scan has stopped measuring anything",
            bodies.len()
        );

        // ★ The two verbs that actually destroy a document. Assembled the same
        // way and for the same reason: spelled as one literal each, they would
        // appear in this test's own body and make every arm look destructive.
        let closes_a_document = |body: &str| {
            let whole = format!("close_{}", "document();");
            let one = format!("close_{}", "slot(");
            body.contains(whole.as_str()) || body.contains(one.as_str())
        };

        let destructive: Vec<&&str> = bodies.iter().filter(|b| closes_a_document(b)).collect();
        assert!(
            destructive.len() >= 2,
            "no arm was found to close a document; the scan is measuring nothing"
        );

        for body in destructive {
            let name = body.split('(').next().unwrap_or("<unnamed>");
            assert!(
                body.contains("ask_unsaved"),
                "`apply_{name}` closes a document without asking about unsaved edits"
            );
            assert!(
                body.contains("save_pending"),
                "`apply_{name}` closes a document without checking `save_pending`"
            );
            // ★ And in that order. Reversed, the operator would be asked a
            // question whose answer cannot be honoured: they press *Close
            // without saving* and are declined anyway, which reads as a broken
            // button rather than as a busy program.
            let pending = body.find("save_pending");
            let ask = body.find("ask_unsaved");
            assert!(
                pending < ask,
                "`apply_{name}` asks about unsaved edits before checking `save_pending`"
            );
        }
    }
}
