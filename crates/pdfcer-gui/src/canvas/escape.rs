//! # `canvas::escape` — the two gestures that must work on a frame that drew nothing
//!
//! ## The dead end this exists to open
//!
//! `OPERATOR_REQUESTS.md` **O186**, stage one, in his own words: *"at some point
//! it sometimes repositions to where the object area I was zooming into is no
//! longer on screen."*
//!
//! When the view ends up somewhere no page overlaps the viewport,
//! `canvas::present` finds nothing in `drawn`, publishes
//! `canvas-unavailable reason=nothing-visible`, and **returns**. That early
//! return sits *above* every input handler in the function, so on such a frame:
//!
//! * the plain wheel does not turn the page,
//! * Ctrl+wheel does not zoom back out,
//! * and there is nothing to click, because there is no page widget to click on.
//!
//! ★★★ **It is terminal.** Measured 2026-09-12: the only ways out are a keyboard
//! route that does not go through the canvas, or closing and reopening the file.
//! A canvas that has stopped responding to the mouse is not a rendering defect
//! the operator can describe — it is *"the program froze"*.
//!
//! `geometry::visible_origin_range` and `deep::confine` stop the
//! view being carried off in the first place, and **this module is still
//! needed**: a clamp protects the route it was written for, and the operator's
//! report lists conditions that were never fully enumerated (*"and at other
//! junctions too"* is the shape this project keeps meeting). A fix for the cause
//! plus a way out if the cause recurs is one decision, not two.
//!
//! ## Why a module, for nine lines
//!
//! ### It is an exception and an exception needs somewhere to state its terms
//!
//! Everything below `present`'s early return assumes **at least one page was
//! drawn**, and several of those assumptions are not checked — the acting page's
//! rect, its extent, the `PageMapping` built from both. ⚠ **So the fix is not to
//! move the `return` lower.** It is to run, above it, the two handlers that need
//! none of that. Naming the exception gives it a place to carry its own
//! justification and a place for the next handler to be added *deliberately*
//! rather than by someone widening a condition.
//!
//! ### R2, and `present` is at 1,381 of 1,500
//!
//! The second reason and not the first, but it is real.
//!
//! ## What is deliberately NOT offered here
//!
//! Anything that needs to know **where** the pointer is on a page: selection,
//! the hit test, a drag, a context menu, the rulers' readout. By construction
//! nothing is hittable on this frame, so a hit test would answer "nothing" and a
//! context menu would open on a page that is not on screen. The two handlers
//! here are exactly the two that are about **the view** rather than about
//! **content**, which is also why neither needs a `Response` of its own.
//!
//! ★ And the gate is `content_hovered` alone, not `… || image_response.hovered()`
//! as the ordinary call sites spell it. There is no acting page on this frame, so
//! there is no second response to consult; the scroll area's own content response
//! still respects layer order, so a floating window over the canvas keeps
//! swallowing the wheel exactly as it does one tier up.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::{paging, zoom};

/// **Run the view-level gestures on a frame that drew no page.**
///
/// Called from `canvas::present` immediately before the
/// `canvas-unavailable reason=nothing-visible` trace and the early return.
///
/// `hovered` is the scroll area's content response — *"is the pointer over the
/// canvas?"* — and is the only gate either handler gets. See the module header.
///
/// # ★ Order matters, and it is the same order the ordinary path uses
///
/// The page turn is read **before** the zoom, so the two are consulted in the
/// order egui produced the events. They cannot both fire on one gesture: a
/// modified wheel populates `zoom_delta` and contributes nothing to the scroll
/// delta [`paging::flip`] reads. Reversing them here would make the rescue path
/// behave differently from the ordinary one under a gesture that is ambiguous on
/// some pointing device nobody has tested — and the whole value of this module
/// is that the operator's habits keep working when the canvas has gone blank.
pub(super) fn offer(ui: &egui::Ui, doc: &mut OpenDoc, hovered: bool, actions: &mut Vec<Action>) {
    paging::flip(ui, doc, hovered, actions);
    if hovered {
        zoom::wheel_step(ui.ctx(), doc, actions);
    }
}
