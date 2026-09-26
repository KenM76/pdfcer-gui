//! # `canvas::widgetdrag` — dragging a form field's box to where it belongs
//!
//! The third module on the annotation branch of [`crate::canvas::dragroute`]'s
//! fork. [`crate::canvas::dimdrag`] answers for a ce dimension,
//! [`crate::canvas::annotdrag`] for ordinary markup, and this for a **form
//! field's widget** — the box an operator types into.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/widgetdrag.md`.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. D5 — Shift constrains to one axis —
//! is applied above this module in [`crate::canvas::dragroute`], so all four
//! verbs on that fork receive one already-constrained delta from one filter.

use egui::Rect;

use crate::app::actions::{Action, forms::FieldAction};
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;

/// The trace line a committed move writes.
// ui-text-exempt: diagnostic trace name, never displayed
const TRACE: &str = "widget-drag";

/// One frame of a widget drag.
pub struct Frame {
    /// The pointer's travel since the press, in canvas space, already
    /// constrained by Shift if it is held.
    pub delta: egui::Vec2,
    /// Where the gesture is.
    pub phase: Phase,
}

/// The selected widget's box, in canvas space, when one is draggable.
#[must_use]
pub fn grab_box(ctx: &egui::Context, doc: &OpenDoc, map: &PageMapping) -> Option<Rect> {
    let selected = doc.selected_field.as_ref()?;
    let placed = crate::canvas::forms::placed(ctx, doc);
    let target = placed.targets.iter().find(|t| {
        t.page == selected.page && t.field == selected.field && t.widget == selected.widget
    })?;
    Some(map.rect_to_screen(target.rect))
}

/// Drive one frame of the drag.
pub fn drag(
    frame: &Frame,
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Option<Rect> {
    let selected = doc.selected_field.as_ref()?;
    let placed = crate::canvas::forms::placed(ctx, doc);
    let target = placed.targets.iter().find(|t| {
        t.page == selected.page && t.field == selected.field && t.widget == selected.widget
    })?;

    if frame.phase != Phase::Complete {
        return Some(target.rect.translate(frame.delta));
    }

    let page = doc.pages.get(selected.page)?;
    let d = super::moving::page_delta(frame.delta, page)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "{TRACE} field={} widget={} dx={:.3} dy={:.3}",
            selected.field, selected.widget, d.dx, d.dy
        )
    });
    // A zero delta is sent rather than filtered, for `annotdrag`'s reason:
    // the engine accepts one by name, and filtering here would mean this shell
    // deciding from a float comparison that an operator's gesture was not one.
    actions.push(Action::Field(FieldAction::MoveWidget {
        field: selected.field.clone(),
        widget: selected.widget,
        dx: d.dx,
        dy: d.dy,
    }));
    None
}
