//! # `app::actions::crossdoc` — pages dragged out of one open document and
//! into another
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/crossdoc.md`.

use crate::app::PdfcerApp;
use crate::app::state::Status;

impl PdfcerApp {
    /// `Action::InsertPagesFromOpenDocument` — take `pages` out of the
    /// document in tab position `source_slot` and put copies of them into the
    /// document on screen, at `position`.
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

        // The parked index, which is NOT the slot.
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
        // Read BEFORE `self.parked` is borrowed mutably below. `Settings`'
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
        // The SOURCE's own disclosures are captured on the way past, and
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
        // `vector_edit` on the SOURCE, which is what buys the whole protocol
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
            // The operator's separation policy, as on every other delete —
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

        // Which sentence, decided by what HAPPENED rather than by what was
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
