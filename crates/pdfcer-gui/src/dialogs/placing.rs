//! # `dialogs::placing` — **how a window offers to step aside**
//!
//! `OPERATOR_REQUESTS.md` **O66**:
//!
//! > *"anything we are inserting like this should have an option in its
//! > dialogue box to place it with the mouse instead of by positional
//! > co-ordinates."*
//!
//! The dialog half of [`crate::canvas::placing`]. That module owns the gesture
//! and the pending record; this owns the button, the note, and the one derived
//! predicate that makes the whole arm safe.
//!
//! ## Opting in is three edits and no new state
//!
//! ```ignore
//! pub struct MyDialog {
//!     place: PlaceHandoff,          // 1. a field
//!     // …
//! }
//!
//! // 2. in the body, beside the numeric controls
//! self.place.button(ui, PlaceKind::MyThing, REGION_PLACE);
//!
//! // 3. the FIRST line of `show`, before the window is built
//! if self.place.hidden(ctx, PlaceKind::MyThing) { return true; }
//! ```
//!
//! ★ Note what the third line does: it returns *"still open"* while drawing
//! nothing. The dialog is not closed — its drafts, its half-typed numbers and
//! its position are all exactly where they were — it simply is not built this
//! frame. That is the difference between stepping aside and being dismissed,
//! and it is the whole reason the operator's numbers survive the trip.
//!
//! ## ★★★ `hidden` is DERIVED, and that is the safety property
//!
//! [`PlaceHandoff`] has **one** field, and it is not the hidden flag. Whether
//! the window is on screen is computed from
//! [`crate::canvas::placing::pending`] every time it is asked.
//!
//! The alternative — a stored `hidden: bool` — is what the precedent this arm
//! generalises actually does, and it is broken. See `canvas::placing`'s header
//! for the Set-scale stranding case in full; the short version is that every
//! route out of a placement (Escape, a mode change, another tool, the document
//! closing) becomes a place somebody has to remember to clear a flag, and one
//! of them was forgotten.
//!
//! With it derived there is nothing to forget. Whatever clears the pending
//! record — including a route written next year by somebody who has never read
//! this file — the window is back on the next frame.

use crate::canvas::placing::PlaceKind;
use crate::text::placing as t;

/// A dialog's offer to step aside, and its one piece of state.
///
/// ★ One field, and it is the **request** rather than the hiding. The request
/// is a genuine edge — the operator pressed a button on this frame and
/// `app::frame` has not seen it yet — so it has to be stored somewhere and
/// read-and-cleared. Being hidden is not an edge; it is a standing
/// consequence of a record that lives elsewhere, so it is asked rather than
/// kept.
#[derive(Default)]
pub struct PlaceHandoff {
    /// Set by [`Self::button`], drained by [`Self::take_request`].
    requested: bool,
}

impl std::fmt::Debug for PlaceHandoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaceHandoff")
            .field("requested", &self.requested)
            .finish()
    }
}

impl PlaceHandoff {
    /// Draw the offer: a button, its tooltip, and the note under it.
    ///
    /// ★ A plain button, never a greyed one. R9 reserves greying for the
    /// *temporarily* unavailable, and this is not unavailable at all — it is
    /// the second of two live routes to the same answer, which is the argument
    /// `dialogs::scale`'s own "Measure it on the drawing…" button already
    /// makes.
    ///
    /// `region` is the dialog's own published rect name, passed in rather than
    /// derived, because a region name is part of the application's published
    /// vocabulary and belongs with the surface that owns it.
    pub fn button(&mut self, ui: &mut egui::Ui, region: &'static str) {
        let button = ui
            .button(t::place_button())
            .on_hover_text(t::place_tooltip());
        crate::diag::ui_rect(region, button.rect);
        if button.clicked() {
            self.requested = true;
        }
        ui.label(egui::RichText::new(t::place_note()).small().weak());
    }

    /// Take the operator's request, if they made one this frame.
    ///
    /// Read-and-clear. A request left set would re-arm the placement on the
    /// frame after the window came back, which the operator would experience as
    /// a dialog that will not stay open.
    pub fn take_request(&mut self) -> bool {
        std::mem::take(&mut self.requested)
    }

    /// **Whether the window should draw nothing this frame** — derived, never
    /// stored.
    ///
    /// See this module's header, and `canvas::placing`'s, for why this is a
    /// question rather than a field.
    #[must_use]
    pub fn hidden(&self, ctx: &egui::Context, kind: PlaceKind) -> bool {
        crate::canvas::placing::pending(ctx).map(|p| p.kind) == Some(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **Clearing the pending record un-hides the window, whatever
    /// cleared it.**
    ///
    /// The property the whole design rests on, and the one a stored flag would
    /// not have. The test deliberately cancels through `canvas::placing` — a
    /// module this one does not otherwise touch — because that is the point:
    /// the un-hiding is not something `PlaceHandoff` participates in.
    #[test]
    fn hidden_follows_the_pending_record_and_nothing_else() {
        let ctx = egui::Context::default();
        let handoff = PlaceHandoff::default();

        assert!(
            !handoff.hidden(&ctx, PlaceKind::Image),
            "nothing pending, so nothing hidden"
        );

        crate::canvas::placing::arm(&ctx, PlaceKind::Image, 0);
        assert!(handoff.hidden(&ctx, PlaceKind::Image), "armed, so hidden");

        crate::canvas::placing::cancel(&ctx);
        assert!(
            !handoff.hidden(&ctx, PlaceKind::Image),
            "★ cancelled through a module this one never calls — and the window is back, \
             because its absence was never a fact of its own"
        );
    }

    /// A window is hidden only by a placement of ITS OWN kind.
    ///
    /// Load-bearing the day there are two: an image placement must not blank
    /// the window that asked for something else.
    #[test]
    fn a_placement_hides_only_the_window_that_asked_for_it() {
        let ctx = egui::Context::default();
        let handoff = PlaceHandoff::default();
        crate::canvas::placing::arm(&ctx, PlaceKind::Image, 0);
        assert!(handoff.hidden(&ctx, PlaceKind::Image));
        // With one variant this is all that can be asserted today; the shape is
        // what matters, and a second kind added later fails here if the
        // comparison is ever loosened to `is_some()`.
        assert_eq!(
            crate::canvas::placing::pending(&ctx).map(|p| p.kind),
            Some(PlaceKind::Image)
        );
    }

    /// The request is an edge and is drained once.
    #[test]
    fn a_request_fires_once() {
        let mut handoff = PlaceHandoff { requested: true };
        assert!(handoff.take_request());
        assert!(
            !handoff.take_request(),
            "a request left set re-arms the placement the frame after the window returns"
        );
    }
}
