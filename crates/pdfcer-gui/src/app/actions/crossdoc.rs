//! # `app::actions::crossdoc` — pages dragged out of one open document and
//! into another
//!
//! One arm, and it is here rather than in [`super::pages`] because it is the
//! only edit in the application that reads **two documents at once**. Every
//! other verb takes a `&mut OpenDoc` and is done; this one needs a
//! `DocumentView` over a *parked* session while it holds `&mut` on the active
//! one, which is a different shape with a different set of things that can go
//! wrong.
//!
//! ---
//!
//! ## 1. The gesture
//!
//! The operator presses on a page tile in the Pages panel, drags — spring-
//! loading a document tab on the way if the destination is another document
//! ([`crate::app::doctabs`] §3) — and releases over the destination's page
//! list or page view at a caret between two sheets. See [`crate::pagedrag`]
//! for the state that survives the document switch in the middle.
//!
//! ## 2. ★ It is a COPY, and the reason is undo
//!
//! [`crate::text::doctabs::drag_landing_other`] carries the argument in the
//! words the operator reads. The engineering form:
//!
//! A cross-document *move* is two edits — an insert into the target and a
//! delete from the source — recorded on **two independent undo stacks**,
//! because `EditSession` owns one command log per document and this
//! application has one session per document. There is no ordering of those two
//! commands under which a single Ctrl+Z means *"undo what I just did"*: undo
//! goes to whichever document has focus, so the operator gets half of their
//! gesture reversed and no indication that the other half is still applied.
//!
//! Half-undone is worse than not-undone, and much worse on a drawing set,
//! where the evidence is a page count nobody checks.
//!
//! Windows Explorer reaches the same conclusion from a different direction and
//! copies between volumes by default. Acrobat's Insert Pages is a copy. So this
//! is a copy, and the caption says so before the operator releases the button.
//!
//! ## 2b. ★★ And Shift makes it a move, which is the operator's call
//!
//! Requested 2026-08-20: *"can you also make it so you can move the pages
//! between documents instead of just copy by holding one of the keys like shift
//! or control, whichever on windows uses to switch from copy to move
//! operation."*
//!
//! **Shift.** Windows has bound the drag modifiers the same way since the
//! mid-nineties — Ctrl copies, Shift moves, Ctrl+Shift makes a shortcut — and
//! [`crate::text::doctabs::drag_landing_move`] carries the table. Copy is
//! already what an unmodified cross-document drag does here, so Ctrl asks for
//! what it already gets and Shift is the modifier that changes the verb.
//!
//! Everything §2 says about undo is still true, so **it is disclosed rather
//! than designed away**: [`crate::text::doctabs::moved_out_of`] states in words
//! that one Ctrl+Z reverses one half of a move, on the status row, immediately
//! after it happens. That is the honest shape — the operator asked for a
//! capability whose cost is real, so they get the capability *and* the cost,
//! rather than a refusal that protects them from a choice that is theirs.
//!
//! ### The order is insert, then delete, and it cannot be the other way
//!
//! The source's pages are removed **only if the target's insert actually
//! happened**, which is why [`super::pages::insert_from_view`] returns a count
//! rather than nothing. Deleting first — or deleting regardless — would lose
//! the operator's sheets to a refusal they never saw: a certified target, an
//! encrypted one, a page tree that will not walk. All three are reachable and
//! all three decline silently as far as the source document is concerned.
//!
//! And if the *delete* is the half that fails, the pages are now in **both**
//! documents. That is a third state, neither of the two things anybody asked
//! for, and [`crate::text::doctabs::move_left_the_source_alone`] says so
//! plainly. Silence there would leave an operator believing they had moved
//! something they had duplicated.
//!
//! ## 3. What the source document is guaranteed **when it is a copy**
//!
//! **Nothing is written to it and nothing is read out of it destructively.**
//! The engine takes a `DocumentView` — a read-only projection — and copies
//! every object it needs at fresh object numbers in the *target*. The source's
//! `EditSession` is not borrowed mutably, its undo stack is untouched, its
//! `is_modified` answer does not change, and its tab does not acquire the
//! unsaved marker.
//!
//! That is worth stating because it is the property that makes the gesture
//! safe to try. An operator who drags the wrong sheet has changed one document
//! and can undo it there.
//!
//! ## 4. What does not come across, and why the operator is told at the moment
//! it happens
//!
//! Exactly what [`super::pages::insert_from_view`] reports for an insert from
//! a file, through the same [`crate::text::pages::inserted`] sentence: page
//! content, resources, fonts and XObjects arrive; the source's **document-level**
//! structures — outlines, the AcroForm field tree, named destinations, page
//! labels — do not, because merging those rewrites objects an incremental save
//! exists in order not to touch.
//!
//! R8b rule 4's surviving half is the reason this is disclosed rather than
//! left to be discovered: *"inferences the operator cannot see … still owe an
//! off-canvas report"*. A form field whose widget arrived without its
//! definition looks exactly like a form field until it is filled in.

use crate::app::PdfcerApp;
use crate::app::state::Status;

impl PdfcerApp {
    /// `Action::InsertPagesFromOpenDocument` — take `pages` out of the
    /// document in tab position `source_slot` and put copies of them into the
    /// document on screen, at `position`.
    ///
    /// **The target is always the active document**, and is not carried by the
    /// action. That is not an omission: the drop landed on a surface, the
    /// surface was showing the active document, and a slot carried from the
    /// press would name whatever was active when the *drag started* — which,
    /// with spring-loading, is precisely the document it is not.
    ///
    /// # The three ways it declines, and why each is silent or spoken
    ///
    /// | condition | what happens |
    /// |---|---|
    /// | the source tab has gone, or never held an open document | traced, nothing said — unreachable without a close mid-drag, and there is no remedy to offer |
    /// | nothing is open to drop into | traced, nothing said — same |
    /// | the source **is** the target | traced and **refused**; a same-document drag is a reorder and reaches a different arm entirely |
    /// | the engine refuses the insert | `vector_edit`'s own decline path, which puts the engine's reason on the status row |
    pub(super) fn apply_insert_from_open_document(
        &mut self,
        source_slot: usize,
        pages: &[usize],
        position: pdfcer_core::pageops::InsertPosition,
        take: bool,
    ) {
        if source_slot == self.active_slot {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-drop-refused slot={source_slot} reason=same-document"
                )
            });
            return;
        }

        // ★ The parked index, which is NOT the slot.
        //
        // `crate::app::documents` §1: `parked` holds the open documents in tab
        // order **with the active one removed**, so every slot above the
        // active one is one place earlier in the vector. Getting this wrong
        // inserts the wrong document's pages, silently and plausibly, which is
        // why it is written once here rather than at each of the two borrows
        // below.
        let parked_index = if source_slot < self.active_slot {
            source_slot
        } else {
            source_slot.wrapping_sub(1)
        };

        // Two disjoint FIELD borrows, taken as two statements. `self.parked`
        // and `self.status` are different fields, so the borrow checker splits
        // them; routing either through `self.slot(..)` — a method on `&self` —
        // would borrow the whole application and make the second borrow
        // impossible. `crate::app::documents`' header explains why the
        // encoding is two fields at all, and this is the one place that
        // benefits from it.
        let Some(Status::Open(source)) = self.parked.get(parked_index) else {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-drop-refused slot={source_slot} reason=source-not-open"
                )
            });
            return;
        };
        let view = source.session.view();
        let source_path = source.path.clone();

        let Status::Open(target) = &mut self.status else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-drop-refused reason=no-target".to_owned()
            });
            return;
        };

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-drop from={source_path:?} pages={} into={:?} position={position:?}",
                pages.len(),
                target.path,
            )
        });

        // Everything past this point is an ordinary insert, and deliberately
        // the *same* one an Insert-from-file performs. See that function's
        // docs for the disclosure and for why the view goes to what arrived.
        let inserted = super::pages::insert_from_view(target, &view, pages, position);
        if !take || inserted == 0 {
            return;
        }
        // Captured before the borrows end: the move's own sentence is stamped
        // with the TARGET's revision, because the target is the document on
        // screen and the status bar only ever draws the active one's.
        let target_epoch = target.edit_epoch;
        let source_label = crate::text::doctabs::tab_label(&source_path, false);
        self.take_pages_from(source_slot, pages, target_epoch, &source_label);
    }

    /// **The second half of a move** — remove the pages from the document they
    /// came from, now that the first half has demonstrably happened.
    ///
    /// Split out of the arm above because it needs the **opposite borrow**. The
    /// insert holds `&self.parked[i]` together with `&mut self.status`; this
    /// holds `&mut self.parked[i]` and nothing else. Two borrows of one field
    /// that differ in mutability cannot overlap, so they cannot be one
    /// function, and forcing it would mean cloning a document to satisfy the
    /// compiler.
    ///
    /// # Why the disclosure is stamped with the TARGET's epoch
    ///
    /// Because the target is the document on screen. `crate::app::status` draws
    /// the **active** document's disclosure and nothing else, so a sentence
    /// filed against the source's revision would be recorded, correct, and
    /// invisible — the shape of failure `app::actions::vector_edit`'s own
    /// header calls *recorded, not disclosed*.
    fn take_pages_from(
        &mut self,
        source_slot: usize,
        pages: &[usize],
        target_epoch: u64,
        source_label: &str,
    ) {
        // `documents` §1's arithmetic again, and written out again rather than
        // shared: the caller's copy is inside a borrow that has ended by the
        // time this runs, and a helper returning it would be a third place the
        // encoding is known.
        // ★ Read BEFORE `self.parked` is borrowed mutably below. `Settings`'
        // fields are `Copy`, so this is a value rather than a borrow, which is
        // what lets the closure use it while the source document is borrowed.
        let separations = self.settings.separations;
        let parked_index = if source_slot < self.active_slot {
            source_slot
        } else {
            source_slot.wrapping_sub(1)
        };
        let Some(Status::Open(source)) = self.parked.get_mut(parked_index) else {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-move-take-refused slot={source_slot} reason=source-not-open"
                )
            });
            return;
        };
        let before = source.pages.len();
        // ★★ The SOURCE's own disclosures are captured on the way past, and
        // that is not tidiness — it is the only way they reach anybody.
        //
        // `delete_pages` reports what the removal broke: outline items and
        // links that now point at nothing, named destinations that no longer
        // resolve, page labels gone stale. `vector_edit` files those against
        // the document it edited — the SOURCE — and `crate::app::status` draws
        // the **active** document's disclosure and nothing else. So without
        // this they would be recorded, correct, and invisible, on the one
        // document the operator is not looking at.
        //
        // Captured here and re-filed under the target's epoch below, together
        // with the move's own sentence.
        let mut source_notes: Vec<String> = Vec::new();
        // ★ `vector_edit` on the SOURCE, which is what buys the whole protocol
        // for a document that is not on screen: the render worker is cancelled,
        // the mutation goes through `Arc::get_mut`, the epoch is bumped, and
        // `pages::resync` drops the rasters of sheets that have moved.
        //
        // That last one is not ceremony here. A parked document **keeps its
        // page texture and its strip cache** (`crate::app::documents` §4), so
        // skipping the resync would leave pictures of the old page order
        // waiting behind its tab — visible the moment the operator clicks back,
        // and attributable to nothing.
        super::apply::vector_edit(source, "page-move-take", 0, pages.len(), |session| {
            // ★ The operator's separation policy, as on every other delete —
            // this is a page delete wearing a drag's clothes, and a policy that
            // applied on one route and not the other would be the divergence
            // the single funnel exists to prevent.
            let outcome = super::pages::delete(session, pages, separations);
            if let Ok(notes) = &outcome {
                source_notes.clone_from(notes);
            }
            outcome
        });
        let removed = before.saturating_sub(source.pages.len());
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-move-took slot={source_slot} asked={} removed={removed}",
                pages.len()
            )
        });

        // ★ Which sentence, decided by what HAPPENED rather than by what was
        // attempted. `removed == 0` means the insert landed and the delete did
        // not, so the pages are in both documents — a third state neither of
        // the two things anybody asked for, and the one an operator must not
        // discover by counting.
        let mut notes = vec![if removed == 0 {
            crate::text::doctabs::move_left_the_source_alone(source_label)
        } else {
            crate::text::doctabs::moved_out_of(removed, source_label)
        }];
        // The move's own sentence FIRST, then whatever the removal broke in the
        // source. Order is the reading order: *what happened* before *what it
        // cost*, which is the shape every other disclosure in this application
        // uses.
        notes.extend(source_notes);
        super::record_edit_disclosure(Some(super::EditDisclosure {
            epoch: target_epoch,
            notes,
        }));
    }
}
