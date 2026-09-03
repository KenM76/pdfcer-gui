//! # `canvas::widgetdrag` — dragging a form field's box to where it belongs
//!
//! The third module on the annotation branch of [`crate::canvas::dragroute`]'s
//! fork. [`crate::canvas::dimdrag`] answers for a ce dimension,
//! [`crate::canvas::annotdrag`] for ordinary markup, and this for a **form
//! field's widget** — the box an operator types into.
//!
//! ## ★★★ The same defect, one surface along, found by looking for it
//!
//! Ten days after the annotation drag was found to be silently eaten, this was
//! the identical state: a widget could be **selected** on the canvas in Edit
//! mode, its position and size shown in the Properties panel with four numbers
//! and an Apply button — and dragging it did nothing.
//!
//! ⇒ It was found by asking *"where else does this shape exist?"* rather than by
//! waiting for a report, which is the whole value of writing the annotation one
//! up. A class of defect that has been named once is cheap to look for; the same
//! class waiting for an operator to trip over it is not.
//!
//! ★★ And the operator's own instruction that week was *"work on form field
//! editing next and the rest of the features required for editing"*. Four
//! numbers and an Apply button are a form for editing a rectangle. **Dragging is
//! how a person moves a box**, and every program in this class does it — the
//! typed fields are the precise route, not the primary one.
//!
//! ## ★★ Why this is not `annotdrag` with a different id
//!
//! A widget is an annotation — `/Subtype /Widget` — and `move_annotation` would
//! move its `/Rect` perfectly well. The engine **refuses it by name** anyway:
//!
//! > `EditError::AnnotationMoveWrongVerb` … for a **widget** (use
//! > `move_widget(fqn, index, dx, dy)`) … Refused rather than delegated on
//! > purpose: both of those do strictly *more*, and quietly doing less under
//! > this name would give you a second way to move the same thing that silently
//! > produces a worse result.
//!
//! What `move_widget` does more of is the **field**: a widget is addressed by
//! its field's fully-qualified name and an index within it, because one field
//! can draw boxes on three pages and the `/Annots` entry is not the thing an
//! operator renamed. Addressing it by `ObjId` would work and would be a second
//! vocabulary for one subject.
//!
//! ★ So this module exists because the **address** differs, not because the
//! geometry does. That is worth saying plainly: two modules with near-identical
//! bodies are usually one module, and the reason these are two is a fact about
//! the engine's API rather than about dragging.
//!
//! ## Rule 4
//!
//! The ghost is the cursor, which the rule permits by name. Nothing about the
//! widget is tinted or badged, and a filled field renders exactly as it will
//! after a save.
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
///
/// ★★ It reads the **same** target list `canvas::forms` hit-tests and draws, and
/// asks it for the selection's own `(field, widget)`. Three surfaces, one
/// rectangle: what the operator can see, what they can grab, and what moves.
/// `dimdrag::grab_box` and `annotdrag::grab_box` state the identical rule, and
/// it exists because a grab box larger than the drawn outline is a press that
/// works where nothing is shown, and one smaller is an operator missing
/// something they can see.
///
/// ★ The list is cached on `(path, edit_epoch)` by `forms::placed`, so asking
/// every frame costs a map lookup rather than a form walk.
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
///
/// Returns the ghost outline to draw, in **canvas space**, or `None` when there
/// is nothing to draw — which covers both *"nothing draggable is selected"* and
/// *"this is the frame that commits"*.
/// ★ No `PageMapping`, unlike [`grab_box`]. A grab box has to be projected to
/// SCREEN space to be hit-tested against a pointer; a ghost is drawn in CANVAS
/// space, which is what the target list already holds. Taking the mapping here
/// would be a parameter used to convert a value into the space it started in.
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
    // ★ A zero delta is sent rather than filtered, for `annotdrag`'s reason:
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
