//! `app::actions::selecting` — the two actions that change WHAT IS SELECTED and
//! nothing else
//!
//! Carved out of [`super::action::Action`] and [`super::apply`] on 2026-09-07
//! under R2.
//!
//! ## ★★★ The seam, and it is a statement rather than a size cut
//!
//! Every other variant of `Action` asks the document to change: a move, a
//! restyle, a page insert, a save. **These two change nothing in the file at
//! all.** They set `doc.selection`, which is shell state — no `vector_edit`, no
//! undo entry, no `edit_epoch` bump, nothing a save would write.
//!
//! That is a real boundary and it was already implicit: `SelectObject`'s own
//! doc comment argues at length that a *panel* raises an action rather than
//! writing the selection directly, because a panel body is handed `&OpenDoc`
//! and not `&mut`. The argument is about the same property this module is named
//! for, and it now has somewhere to live that is not a file about applying
//! edits.
//!
//! ## ★★ Why now, and the honest version of it
//!
//! `file.import_text` was wired the same day and took `action.rs`, `apply.rs`
//! and `dispatch.rs` **all past R2's 1,500-line ceiling in one commit**. All
//! three were already within twenty lines of it and `RESUME.md` had said so by
//! name — *"one added line in either fails the build. Split before adding, not
//! after."*
//!
//! ⇒ So this split is late rather than clever. What makes it a seam rather than
//! a cut is the paragraph above; what makes it *this* seam rather than another
//! is that these were the only two arms in `apply.rs`'s match that never touch
//! the document, which is the shortest true sentence about any group in that
//! file.

use crate::app::state::OpenDoc;
use pdfcer_core::vector::MarqueeMode;

/// **What is selected**, as an action a panel can raise.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionAction {
    /// **Everything on the page, wherever it now sits** — `edit.select_all`.
    ///
    /// The operator, 2026-09-01: *"we should be able to select things off the
    /// side of the page, especially since I sometimes drop objects there, and
    /// when I do I can't get them back."* The whole argument is on the arm in
    /// [`apply_action`], where the marquee's `Enclosed` mode and the
    /// deliberately unbounded rectangle are.
    SelectAllOnPage,
    /// ★★★ **Select exactly this object** — raised by the Objects panel when a
    /// row is clicked.
    ///
    /// # Why a panel raises an action instead of writing the selection
    ///
    /// Because a panel body is handed `&OpenDoc`, not `&mut`, and that is
    /// deliberate: a surface that could mutate the document while it is being
    /// drawn is a surface that can change what a later widget in the same frame
    /// is describing. Every other panel that changes something raises an action
    /// for the same reason, and this is not the place to make an exception.
    ///
    /// # What it replaced
    ///
    /// `PanelsState::focus` — a second notion of *"the thing I am working on"*,
    /// written only by the Objects panel and read only by the Properties panel,
    /// which the canvas neither wrote nor read. The audit of 2026-08-26 found
    /// three such notions in parallel (the armed tool, the panel focus, the
    /// canvas selection) with no bridge between them, and named it the cause of
    /// the operator's *"when I have an object selected like text the Tool tab
    /// doesn't switch to giving me the editable stuff for that object."*
    ///
    /// Now there is one, written from both ends.
    SelectObject {
        /// The page the object is on, in the session's page space.
        page: usize,
        /// Which object, as a paint-order target — or `None` to select
        /// nothing.
        ///
        /// ★ `None` rather than a second variant, because a row click is one
        /// act with one outcome: *this row is now the selection*. Clicking the
        /// already-selected row makes that selection empty, which is what
        /// clicking a selected item does in every list in every application,
        /// and splitting it into Select and Clear would make the caller decide
        /// which act it was performing when it only ever performs one.
        object: Option<crate::canvas::target::TargetId>,
    },
}

/// **Route one selection action.**
///
/// ★ It takes `&mut OpenDoc` and nothing else — no `PdfcerApp`, no
/// `&mut Vec<Action>`, no preferences. That narrow signature is the module's
/// header made mechanical: an action that could reach anything else would be
/// one that could change something, and neither of these can.
pub(super) fn apply_action(doc: &mut OpenDoc, action: SelectionAction) {
    match action {
        // ★★ A row click in the Objects panel, arriving as an action for
        // the reason `SelectionAction::SelectObject`'s own docs give: a panel body
        // holds `&OpenDoc`, not `&mut`, so a panel that changes something
        // asks rather than writes.
        //
        // Here rather than before the document guard, because it needs the
        // document and has no reason to run without one — the pre-guard
        // match is for the actions that *make* a document open.
        //
        // ★ No `vector_edit`, no epoch bump, no cache invalidation: **a
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
            // ★★★ A LARGE FINITE RECT, not `Rect::EVERYTHING`.
            //
            // The first version used `EVERYTHING` and selected **nothing**,
            // measured on the operator's own drawing: `select-all page=0
            // n=0`. The provider maps the query rectfrom canvas space into
            // PDF space before asking the engine, and an infinite rect put
            // through an affine transform yields NaN — after which every
            // containment test is false and the answer is silently empty.
            //
            // ⇒ Infinity is not a safe "everything" when a coordinate
            // system change stands between the caller and the comparison.
            // A million points is about 350 metres of paper; no page
            // approaches it, and every arithmetic step stays finite.
            const EVERYWHERE: f32 = 1.0e6;
            let all = egui::Rect::from_min_max(
                egui::pos2(-EVERYWHERE, -EVERYWHERE),
                egui::pos2(EVERYWHERE, EVERYWHERE),
            );
            let hits = doc
                .page_objects()
                // ★ `Enclosed` stated rather than implicit, as of 2026-09-02
                // when the mode became a parameter (O88): under
                // `Rect::EVERYTHING` the two modes agree, and a reader must
                // not have to work that out before believing Select All is
                // unaffected by a change to what a rubber band means.
                .map(|p| p.hit_test_rect(page, all, MarqueeMode::Enclosed))
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
