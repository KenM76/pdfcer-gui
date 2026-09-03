//! # `app::actions::reorder` — putting a page's annotations in a new order
//!
//! One verb, split out of [`super::forms`] on 2026-09-02 under R2 when that file
//! crossed the 1,500-line ceiling. `OPERATOR_REQUESTS.md` O99.
//!
//! ## ★★ Why it is worth its own file rather than a shorter comment
//!
//! Because the interesting part is not the call — it is one line — but the
//! **three things the operator did not ask for** and which the engine reports so
//! they can be said. A tab order is a list of *fields*; `/Annots` order is more
//! than that, and every one of the three below is a consequence an operator
//! would not predict from the gesture they made.

use crate::app::state::OpenDoc;

/// **Put a page's annotations in a new order** — `OPERATOR_REQUESTS.md` O99.
///
/// # ★★★ Three disclosures, and two of them are about things the operator did
/// not ask for
///
/// A tab order is a list of *fields*. `/Annots` order is more than that, and the
/// engine reports the difference rather than letting it happen quietly:
///
/// * **`non_widgets_moved`** — `/Annots` order is **paint order** for every
///   annotation, so moving a widget past a `/Link` or a markup changes which is
///   drawn on top where they overlap. The operator arranged a tab order and got
///   a z-order change; that has to be said.
/// * **`pinned`** — entries written as direct dictionaries have no object id to
///   be named by, so they cannot be moved and stay at their index while the rest
///   flow around them. A list that did not fully take, disclosed rather than
///   discovered.
/// * **`array_copied`** — the page's `/Annots` was shared with another page and
///   had to be copied first. Nothing is wrong, and it is a structural change to
///   the file that nobody asked for.
///
/// ★★ `moved == 0` is a **success with nothing to say**: the order given was the
/// order the page already had, the engine recorded no command, and there is
/// nothing to disclose. It is the common case for a drag that ends where it
/// started, and it must not read as a refusal.
pub(super) fn reorder_annotations(
    doc: &mut OpenDoc,
    page: usize,
    order: &[pdfcer_core::object::ObjId],
) {
    super::apply::vector_edit(doc, "reorder-annotations", page, 1, |session| {
        session.reorder_annotations(page, order).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                //
                // ★ `moved=` beside `entries=`, because a reorder that moved
                // nothing and a reorder that moved everything produce the same
                // `entries` and want opposite readings.
                format!(
                    "reorder-annotations-applied page={page} entries={} moved={} \
                     non_widgets={} pinned={} copied={}",
                    outcome.entries,
                    outcome.moved,
                    outcome.non_widgets_moved,
                    outcome.pinned,
                    outcome.array_copied
                )
            });
            let mut notes = Vec::new();
            if outcome.non_widgets_moved > 0 {
                notes.push(crate::text::forms::reorder_moved_non_widgets(
                    outcome.non_widgets_moved,
                ));
            }
            if outcome.pinned > 0 {
                notes.push(crate::text::forms::reorder_pinned(outcome.pinned));
            }
            if outcome.array_copied {
                notes.push(crate::text::forms::reorder_copied_shared_array().to_owned());
            }
            notes
        })
    });
}
