//! # `app::actions::view` — **the verbs that move the operator, not the
//! document**
//!
//! Fit, zoom, the three page steps, and a bookmark's destination. Split out of
//! `app::actions::apply` under R2 on 2026-09-01, and the seam is a real one
//! rather than a line count: **every other arm in that match changes a PDF, and
//! none of these does.**
//!
//! ## ★★ Why that distinction is worth a file
//!
//! `apply`'s whole discipline is built around edits — the funnel, the epoch
//! bump, the render-worker cancel, the disclosure channel. None of it applies
//! here, and an arm that needs none of it sitting among ninety that do is an
//! invitation to give it some by accident. A reader asking *"does this bump the
//! epoch?"* gets the answer from the file name.
//!
//! ## ★ The destination is PARKED, not performed
//!
//! Landing on a bookmark's destination needs the canvas rectangle, the page's
//! drawn extent and the scroll offset. None of those exists here, so this
//! records the request and `canvas::destination` drains it on the next frame —
//! `OpenDoc::fit_placement`'s own pattern, for its own reason.

use crate::app::state::OpenDoc;

use super::Action;

/// Apply one view verb.
///
/// The caller has already matched the variant set; the `_` arm is unreachable
/// and says so rather than silently doing nothing — the same rule
/// `app::dispatch::format` states for its own guarded fall-through.
pub(super) fn apply(doc: &mut OpenDoc, action: Action, page_count: usize, max_zoom: f32) {
    match action {
        // ★ A fit sets the scale AND asks for the view to be placed --
        // `OPERATOR_REQUESTS.md` O28. The placement cannot happen here:
        // the re-fitted zoom is computed by `ViewState::apply_fit` from a
        // viewport this code cannot see, so the page's new drawn size is
        // not known until the canvas next runs. So the request is
        // recorded and the canvas spends it, exactly as a discrete zoom
        // records an anchor and the canvas solves it a frame later.
        //
        // `pinned_axes` returns `None` for `FitMode::None`, which changes
        // no zoom and therefore has no new extent to place against --
        // moving the view for it would be a jump for a command that did
        // nothing.
        Action::Fit(mode) => {
            doc.view.set_fit(mode);
            if mode.pinned_axes().is_some() {
                doc.fit_placement = Some(mode);
            }
        }
        Action::ZoomTo(zoom) => doc.view.set_zoom(zoom, max_zoom),
        Action::NextPage => doc.view.next_page(page_count),
        Action::PrevPage => doc.view.prev_page(page_count),
        Action::GoToPage(index) => doc.view.go_to_page(index, page_count),
        // ★★ **A bookmark's destination**, parked rather than performed:
        // the landing needs a viewport, which this phase has none of.
        // `canvas::destination` drains it and carries the argument.
        Action::GoToDestination(to) => doc.pending_destination = Some(to),
        // ui-text-exempt: a panic message, read from a stack trace by whoever
        // widened the caller's variant set without widening this. Never shown.
        other => debug_assert!(false, "{other:?} is not a view verb"),
    }
}
