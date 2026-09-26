//! # `app::actions::document` — the actions that decide WHICH document is on
//! screen, and the two guards the destructive ones share
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/document.md`.

use crate::app::PdfcerApp;
use crate::dialogs::unsaved::PendingIntent;

impl PdfcerApp {
    /// `Action::Open` — open the document at `path`, **in a tab of its own**.
    pub(super) fn apply_open(&mut self, path: std::path::PathBuf) {
        self.open_path(path);
    }

    /// `Action::OpenWithPassword` — the retry an encrypted document needs.
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
