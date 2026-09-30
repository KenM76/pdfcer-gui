//! # `app::dispatch::aligning` — the seven one-click aligns (Ctrl+Alt+keypad).
//!
//! Each presses Align buttons against the panel's current *Relative to* and
//! group toggle, exactly as a press in the panel would; Inkscape binds the
//! same chords to the same buttons. *Centre on both axes* is two presses
//! landed as one undo step.

use pdfcer_gui_base::alignlayout::{Axis, Edge};

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::panels::align::{self, Op};

/// The buttons each command presses.
fn ops(id: &str) -> Option<&'static [Op]> {
    // ui-text-exempt: registered command ids, never displayed.
    Some(match id {
        "edit.align_left" => &[Op::Align(Axis::X, Edge::Min)],
        "edit.align_right" => &[Op::Align(Axis::X, Edge::Max)],
        "edit.align_top" => &[Op::Align(Axis::Y, Edge::Min)],
        "edit.align_bottom" => &[Op::Align(Axis::Y, Edge::Max)],
        "edit.align_centre_x" => &[Op::Align(Axis::X, Edge::Centre)],
        "edit.align_centre_y" => &[Op::Align(Axis::Y, Edge::Centre)],
        "edit.align_centre" => &[
            Op::Align(Axis::X, Edge::Centre),
            Op::Align(Axis::Y, Edge::Centre),
        ],
        _ => return None,
    })
}

/// Dispatch one of the seven.
pub(super) fn dispatch(app: &PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let (Some(ops), Status::Open(doc)) = (ops(id), &app.status) else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("align-chord id={id} ops={ops:?}")
    });
    align::run(doc, ops, app.panels.align, actions);
}
