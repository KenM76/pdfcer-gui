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
pub(super) fn offer(ui: &egui::Ui, doc: &mut OpenDoc, hovered: bool, actions: &mut Vec<Action>) {
    paging::flip(ui, doc, hovered, actions);
    if hovered {
        zoom::wheel_step(ui.ctx(), doc, actions);
    }
}
