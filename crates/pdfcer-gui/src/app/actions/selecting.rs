//! `app::actions::selecting` — the actions that change what is selected and
//! nothing else
//!
//! ## The boundary this module is named for
//!
//! Every other variant of `Action` asks the document to change: a move, a
//! restyle, a page insert, a save. **The variants here change nothing in the
//! file at all.** They set `doc.selection`, which is shell state — no
//! `vector_edit`, no undo entry, no `edit_epoch` bump, nothing a save would
//! write.
//!
//! The boundary is what the narrow signature of [`apply_action`] enforces:
//! taking `&mut OpenDoc` and nothing else means an arm added here cannot reach
//! anything it would have to reach in order to edit. Keep it that way — an arm
//! that needs `PdfcerApp` belongs in a sibling module, because needing it is
//! the evidence that the arm is not purely a selection.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/selecting.md`.

use crate::app::state::OpenDoc;
use pdfcer_core::vector::{FormMarquee, MarqueeMode};

pub use pdfcer_gui_base::subactions::SelectionAction;

/// **Route one selection action.**
pub(super) fn apply_action(doc: &mut OpenDoc, action: SelectionAction) {
    match action {
        // Routed after the document guard, because a selection names parts of
        // an open document and has no meaning without one — the pre-guard
        // match is for the actions that *make* a document open.
        //
        // No `vector_edit`, no epoch bump, no cache invalidation: **a
        // selection is not an edit.** It names parts of a document and
        // changes nothing a save would write. `canvas`'s header makes that
        // argument for the canvas selection; this is the same argument
        // arriving from the other end of the same selection.
        SelectionAction::SelectObject { page, object } => match object {
            Some(object) => doc.selection.select_only(page, object, "objects-panel"),
            None => {
                doc.selection.clear();
            }
        },
        SelectionAction::SelectAllOnPage => {
            let page = doc.view.page_index;
            // A large finite rect, never `Rect::EVERYTHING`.
            //
            // The provider maps the query rect from canvas space into PDF
            // space before asking the engine, and an infinite rect put
            // through an affine transform yields NaN — after which every
            // containment test is false and Select All silently answers
            // empty. Infinity is not a safe "everything" when a coordinate
            // system change stands between the caller and the comparison.
            //
            // A million points is about 350 metres of paper; no page
            // approaches it, and every arithmetic step stays finite.
            const EVERYWHERE: f32 = 1.0e6;
            let all = egui::Rect::from_min_max(
                egui::pos2(-EVERYWHERE, -EVERYWHERE),
                egui::pos2(EVERYWHERE, EVERYWHERE),
            );
            let hits = doc
                .page_objects()
                // Both modes stated rather than left to a default, so that a
                // change to what a rubber band means cannot silently change
                // what Select All answers.
                //
                // `Include`, because Select All is a census: a form on the
                // page is one of the things on the page and is an edit
                // operand, so leaving it out would make *"select everything,
                // then delete"* quietly leave the title block behind.
                .map(|p| p.hit_test_rect(page, all, MarqueeMode::Enclosed, FormMarquee::Include))
                .unwrap_or_default();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("select-all page={page} n={}", hits.len())
            });
            // `false` — a Select All REPLACES. Extending would make a
            // second press a no-op and a first press after a click keep the
            // click, neither of which is what the command says.
            doc.selection.marquee(page, &hits, false);
        }
    }
}
