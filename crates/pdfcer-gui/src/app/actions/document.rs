//! # `app::actions::document` — the actions that decide WHICH document is on
//! screen, and the two guards the destructive ones share
//!
//! ## Why this is a file of its own
//!
//! `apply`'s match has a block at the top whose own comment reads: *"The
//! three actions that are about WHICH document is open, matched BEFORE the
//! guard below."* That is the seam this file is cut along. Everything below
//! that block acts **on** the open document;
//! everything in it decides **which** document is open, or whether there is
//! one. That is a different subject with a different failure mode — the arms
//! below can be wrong about a page, and these can be wrong about an afternoon's
//! work.
//!
//! ## Which arms carry the guards, and which must not
//!
//! | arm | discards a document? | guards |
//! |---|---|---|
//! | `apply_open` | **no** — it adds a tab | none |
//! | `apply_open_with_password` | **no** — the same, after a retry | none |
//! | `apply_new` | **no** | none |
//! | `apply_new_sized` | **no** | none |
//! | `apply_close` | yes — the one on screen | both |
//! | `apply_close_document` | yes — the one whose tab was clicked | both |
//! | `apply_close_other_documents` | yes — every other tab | both, by delegation |
//! | `apply_reread_with_duplicate_keys` | **yes** — replaces it with a fresh parse of its own bytes | both |
//!
//! Open and New park what is open and add a tab. Nothing is discarded, so the
//! question a guard on them would ask — *"your unsaved edits will be lost"* —
//! would be **false**, and a confirmation that says something untrue is how an
//! operator learns to dismiss confirmations unread. The protection belongs
//! where the loss happens, and the loss happens in more than one place, which
//! is exactly what
//! [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
//! is shaped to notice.
//!
//! ## The two guards, in order, and why the order is not interchangeable
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
//! ## The failure mode this file's shape closes
//!
//! The two guards look alike from a distance, and an arm that has one reads as
//! an arm that is protected. `file.close`'s tooltip promises the operator
//! *"You are asked what to do about unsaved edits first."* A guard that is
//! present, well argued, correct, **and answering the other question** looks
//! exactly like the guard that promise needs — which is how an arm comes to
//! destroy every edit made since the file was opened, silently, with a doc
//! comment above it explaining the guard it does have. Keeping every such arm
//! in one file, under the table above, is what makes the mismatch visible.
//!
//! ## An arm can discard a document without closing anything
//!
//! `apply_reread_with_duplicate_keys` is the arm whose destruction is
//! invisible in its own shape. It does not say *close*. The tab does not go
//! away. The path, the name and the page count are all identical afterwards.
//! What is gone is **every edit made since the file was opened**, because the
//! engine's intervention is a re-load rather than a patch — *"a decision made
//! during parsing is not a value that can be edited afterwards, because the
//! discarded one was never built into the document"* — so the document that
//! comes back is a fresh parse of the bytes on disk with an empty undo stack.
//!
//! That is the same failure mode from a new direction: **a reader has no
//! reason to ask whether this arm needs the guards**, because nothing about it
//! looks destructive. It is in this file, in the table above, and inside the
//! test's destructive set for that reason and no other — the test's predicate
//! names the re-read verb as well as the two close verbs, because *"names a
//! close verb"* is only a proxy for *discards a document*.
//!
//! [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
//! is the half of this that fails when a new arm arrives without the guards.
//! It asserts the property rather than a count of arms, because the count was
//! never what mattered.

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
    /// **It asks nothing about unsaved edits, and that is deliberate.**
    ///
    /// `open_path` parks what was open and adds a tab. Nothing is discarded, so
    /// there is nothing to ask about — and asking anyway would be worse than
    /// useless, because the question *"Open another document? Your unsaved
    /// edits will be lost."* would be **false**.
    ///
    /// The reasoning a guard here would rest on is worth keeping in view,
    /// because it is the strongest case for one anywhere in the file: an
    /// operator who has marked up a drawing and then opens the next one
    /// destroys exactly as much work as one who pressed Close, and is far more
    /// likely to do it, because opening the next file is what you do all day
    /// whereas closing a document deliberately is something you do at the end
    /// of one. That case is answered where the loss actually happens, which is
    /// a close.
    ///
    /// The `save_pending` guard is absent for the same reason: it means
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
    /// The password is **borrowed**, never cloned into a second place. It
    /// arrives inside the action, is handed to `Document::load_with_password`
    /// through `Secret::expose`, and the action is dropped with it. There is no
    /// step here that stores it.
    pub(super) fn apply_open_with_password(
        &mut self,
        path: std::path::PathBuf,
        password: &crate::secret::Secret,
    ) {
        match self.open_path_with_password(path, password) {
            // The prompt stays up and says WHICH failure it was. The two are
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
    /// Unguarded, for [`Self::apply_open`]'s reason: it adds a document rather
    /// than replacing one.
    pub(super) fn apply_new(&mut self) {
        self.new_document();
    }

    /// `Action::NewSized` — the same, with a page box the operator chose.
    ///
    /// Beside the plain New and unguarded with it, for [`Self::apply_open`]'s
    /// reason. The two are kept adjacent and identical in shape so that a
    /// change to what either guard means cannot be applied to one New and
    /// missed on the other.
    pub(super) fn apply_new_sized(&mut self, width_pt: f64, height_pt: f64) {
        // The lower-left corner is the origin: a new page has nothing to offset
        // from, and `Action::NewSized`'s own docs say why the action carries a
        // size rather than a rectangle.
        self.new_document_sized(pdfcer_core::page_tree::Rect::from_corners(
            0.0, 0.0, width_pt, height_pt,
        ));
    }

    /// `Action::RereadWithDuplicateKeys` — **read this file again, taking the
    /// other value wherever it names a key twice.**
    ///
    /// The operator's own intervention in a parse decision:
    ///
    /// > *"We should be making pdfcer so that it opens pdfs that have errors,
    /// > and have a way that it manages those errors such that they aren't
    /// > fatal, and if the user can intervene in a decision that should always
    /// > be an option along with them not having to intervene."*
    ///
    /// Raised only by `crate::panels::docprops`, from beside the list of places
    /// the file contradicted itself — which is where it has to be raised from,
    /// because the disclosure is the only thing on screen that makes the offer
    /// mean anything. R8b rule 4 puts that disclosure **off-canvas** and this
    /// keeps its control there with it.
    ///
    /// # Why it carries both guards when nothing about it says *close*
    ///
    /// Because it destroys as much as a Close does and advertises none of it.
    /// The tab stays, the path stays, the pages look identical — and every edit
    /// since the file was opened is gone, because the engine's intervention is
    /// a re-load: *"the discarded one was never built into the document"*. An
    /// operator who has marked up a drawing and then presses *use the first
    /// value* out of curiosity has lost the afternoon.
    ///
    /// So: same two questions, same order, same reasons as
    /// [`Self::apply_close`]. This file's header carries the argument for the
    /// order and it transfers here without amendment.
    ///
    /// # The intent is `Reread`, not `Close`, and that is load-bearing
    ///
    /// [`PendingIntent::Reread`] carries the operator's chosen
    /// [`pdfcer_core::document::LoadOptions`] **across the dialog**. Resuming
    /// through `PendingIntent::Close` would close the tab and stop — the
    /// operator would answer a question about their edits and watch their
    /// document vanish instead of coming back read the way they asked. Every
    /// step of the chain has to hold the reading, and this is the step where it
    /// would be easiest to drop.
    ///
    /// # It does not validate `policy`, and one value must never reach it
    ///
    /// [`pdfcer_core::parser::DuplicateKeyPolicy::Refuse`] is representable in
    /// the action and is that enum's own `Default`. Sent to a loader it is the
    /// behaviour that **refuses a 46 KB drawing whole over one repeated
    /// `/PageMode`** — the failure this whole offer exists to end.
    /// The rule is enforced where the value is constructed, in
    /// `crate::panels::docprops`, which offers two of the three; this arm is
    /// the transport and cannot second-guess a policy the engine may extend.
    pub(super) fn apply_reread_with_duplicate_keys(
        &mut self,
        policy: pdfcer_core::parser::DuplicateKeyPolicy,
    ) {
        // As `apply_close`: an intervention the operator started themselves
        // ends any `Close others` sequence that was waiting for an answer.
        self.closing_others = None;
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "reread-declined reason=save-pending".to_owned()
            });
            return;
        }
        let options = pdfcer_core::document::LoadOptions::new().with_duplicate_keys(policy);
        if self
            .dialogs
            .ask_unsaved(&self.status, PendingIntent::Reread { options })
        {
            // The question is up. `PdfcerApp::resume_after_unsaved` finishes
            // this, with the same `options` — that is what the intent carries
            // them for.
            return;
        }
        // Nothing at stake, so it happens now. The `false` return — no file
        // behind this document — is traced by the callee and needs no handling
        // here: the control that raises this action is drawn only over a
        // document that reported load anomalies, and a document with no bytes
        // cannot have reported any.
        let _ = self.reread_active_document(options);
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
    /// It asks the same two questions in the same order as
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
        // O65: `save::has_unsaved_edits`, not `session.is_modified()`. The
        // engine's answer is "differs from the BASE revision", which an
        // incremental save cannot clear — so a **saved** background tab reads
        // as modified, which yanks the canvas to it (`activate_slot` below) to
        // ask a question that should not be asked at all.
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
    /// **It has both guards by DELEGATION** rather than by carrying its own
    /// copies. Every close it performs goes through
    /// [`Self::apply_close_document`], which is where the guards live, so there
    /// is no second place for *"does this ask about unsaved edits?"* to be
    /// answered differently.
    ///
    /// It is also why [`tests::every_action_that_discards_a_document_asks_about_unsaved_edits`]
    /// holds without listing this arm: the body names a close verb only through
    /// its sibling, and the sibling is checked.
    ///
    /// # It closes from the RIGHT, and `keep` is adjusted as it goes
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
    /// # It survives the unsaved-edits question, which is what makes it
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
    /// **Every action here that DISCARDS a document asks about unsaved edits,
    /// in the right order.**
    ///
    /// # What is asserted, and what deliberately is not
    ///
    /// Not *"there are N arms in this file and all of them call
    /// `ask_unsaved`"*. A count is not the property, and an arm that adds a tab
    /// rather than replacing one must **not** ask — a guard there would state
    /// something false to the operator, so a test keyed on the count would
    /// demand a lie the moment the count moved.
    ///
    /// The property is **an arm that can destroy a document must ask first**,
    /// and that is what is checked: any body naming a destructive verb must
    /// also name both guards, in order. The counts survive only as floors — an
    /// instrument that cannot fail detects nothing — and they are floors on
    /// *both* populations, so neither "no arms were found" nor "no destructive
    /// arms were found" can pass silently.
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
        // The marker is ASSEMBLED from two pieces rather than written as one
        // literal, and it has to be.
        //
        // The scan looks for the function signatures. Writing that signature
        // out as a single string — here, or in a comment explaining why not to
        // — puts an extra copy of it into the very file being scanned, and the
        // split finds one body too many.
        //
        // **The instrument would then be counting itself**, and the spurious
        // body would contain `ask_unsaved` and `save_pending` — they appear in
        // the assertion messages — so it would pass every check below. A
        // source-scanning test is part of its own corpus; only the floor
        // assertion notices when that stops being accounted for.
        let marker = format!("    pub(super) {}", "fn apply_");
        let bodies: Vec<&str> = SRC.split(marker.as_str()).skip(1).collect();
        assert!(
            bodies.len() >= 6,
            "found {} arms; the scan has stopped measuring anything",
            bodies.len()
        );

        // The verbs that actually destroy a document. Assembled the same way
        // and for the same reason: spelled as one literal each, they would
        // appear in this test's own body and make every arm look destructive.
        //
        // **The third verb does not close anything.** "Names a close verb" is
        // only a **proxy** for the property being asserted, which is *"this arm
        // can destroy the operator's work"*, and
        // `apply_reread_with_duplicate_keys` breaks the proxy: it discards every
        // edit in the document and closes nothing — the tab stays, the path
        // stays, and the engine hands back a fresh parse of the same bytes with
        // an empty undo stack. Left out of this list it would be classified
        // **harmless**, the loop would skip it, the arm count would still add
        // up, and the test would go green over an arm that can lose an
        // afternoon.
        //
        // The guard against the next one is not a better name. It is the rule
        // stated here: **when an arm can discard a document, it goes in this
        // list, whatever its verb is called.**
        let closes_a_document = |body: &str| {
            let whole = format!("close_{}", "document();");
            let one = format!("close_{}", "slot(");
            let again = format!("reread_active_{}", "document(");
            body.contains(whole.as_str())
                || body.contains(one.as_str())
                || body.contains(again.as_str())
        };

        let destructive: Vec<&&str> = bodies.iter().filter(|b| closes_a_document(b)).collect();
        assert!(
            destructive.len() >= 3,
            "found {} arms that discard a document; the scan is measuring less \
             than it did on 2026-09-10",
            destructive.len()
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
            // And in that order. Reversed, the operator would be asked a
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
