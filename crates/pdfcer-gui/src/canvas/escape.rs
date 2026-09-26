//! # `canvas::escape` — the two gestures that must work on a frame that drew nothing
//!
//! ## The dead ends this exists to open
//!
//! `canvas::present::show_in` has **two** early returns that publish a
//! `canvas-unavailable` trace and leave, and both sit *above* every input
//! handler in the function. On a frame taking either of them:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/escape.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::{paging, zoom};

/// **Run the view-level gestures on a frame that drew no page.**
///
/// Called from `canvas::present` immediately before each of the two
/// `canvas-unavailable` traces and their early returns.
///
/// `hovered` answers *"is the pointer over the canvas?"* and is the only gate
/// either handler gets; which response the caller reads it from is a property
/// of the call site. See the module header.
///
/// # Order matters, and it is the same order the ordinary path uses
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
