//! # `canvas::annotdrag` — dragging a markup annotation to where it belongs
//!
//! The other half of the annotation-drag fork. [`crate::canvas::dimdrag`]
//! answers for a **ce dimension**; this answers for everything else pdfcer puts
//! on a page — a stamp, an ink stroke, a callout box, a highlight, a note.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotdrag.md`.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. D5 (Shift constrains to one axis) is
//! applied **above** this module, in `canvas::interact`, so both branches of the
//! annotation fork and the content branch all receive one already-constrained
//! delta from one filter. A second copy of that rule here is how two drags come
//! to disagree about what Shift means.

use egui::Rect;

use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{AnnotKind, SelectionState};
use pdfcer_core::page_tree::Page;

/// The trace line a committed move writes.
// ui-text-exempt: diagnostic trace name, never displayed
const TRACE: &str = "annot-drag";

/// One frame of an annotation drag.
pub struct Frame {
    /// The pointer's travel since the press, in canvas space, already
    /// constrained by Shift if it is held.
    pub delta: egui::Vec2,
    /// Where the gesture is.
    pub phase: Phase,
}

/// Whether this selection is one this module moves, and its outline if so.
#[must_use]
fn eligible(selection: &SelectionState) -> Option<(pdfcer_core::object::ObjId, Rect)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::Markup || annot.target.locked {
        return None;
    }
    Some((annot.target.id, annot.outline))
}

/// The screen-space box a press must land in to mean *move this markup*.
#[must_use]
pub fn grab_box(
    map: &crate::canvas::mapping::PageMapping,
    selection: &SelectionState,
) -> Option<crate::canvas::handles::GripFrame> {
    let (_, outline) = eligible(selection)?;
    // **The TURNED frame when there is one** — `OPERATOR_REQUESTS.md` O147.
    //
    // `AnnotSelection::oriented` is `Some` only when the mark's appearance
    // carries a rotation that `/Rect` therefore cannot describe. It is read
    // here, at the one place that answers *"what can be grabbed and where"*, so
    // the painter and the hit test cannot disagree about it — the pair that
    // `Grabbable`'s own header records going wrong on 2026-08-20.
    Some(selection.annot().and_then(|a| a.oriented).map_or_else(
        || crate::canvas::handles::GripFrame::Upright(map.rect_to_screen(outline)),
        |quad| crate::canvas::handles::GripFrame::Turned(quad.map(|p| map.to_screen(p))),
    ))
}

/// Drive one frame of the drag.
pub fn drag(
    frame: &Frame,
    page: Option<&Page>,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> Option<Rect> {
    let (id, outline) = eligible(selection)?;

    if frame.phase != Phase::Complete {
        // The ghost is the selection's own outline, translated. It is the same
        // rectangle `overlay::draw_selection` strokes, which is the property
        // that matters: what the operator grabbed and what they see moving are
        // one rectangle, so a drag cannot appear to pick up something else.
        return Some(outline.translate(frame.delta));
    }

    // --- the commit ---------------------------------------------------------
    // The page arrives as a parameter rather than being read off the
    // document, and it is what lets every rule in this module be tested without
    // a window or a file. `dimdrag` takes `&OpenDoc` because it must scan the
    // dimension model; this needs nothing but a coordinate transform.
    let d = super::moving::page_delta(frame.delta, page?)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{TRACE} id={} dx={:.3} dy={:.3}", id.num, d.dx, d.dy)
    });
    // A zero delta is sent, not filtered out here.
    //
    // The engine accepts one by name — *"a drag that returns to its start
    // should not make you special-case your own arithmetic"* — and filtering it
    // here would mean this shell deciding, from a float comparison, that an
    // operator's gesture was not a gesture. It costs one undo entry for a move
    // of nothing, which is what every drawing program in this class does.
    actions.push(Action::Annot(
        crate::app::actions::annot::AnnotAction::Move {
            id,
            dx: d.dx,
            dy: d.dy,
        },
    ));
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::{AnnotSelection, AnnotTarget};
    use pdfcer_core::object::ObjId;

    fn selection(kind: AnnotKind, locked: bool) -> SelectionState {
        let mut state = SelectionState::default();
        state.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id: ObjId::new(7, 0),
                kind,
                // ui-text-exempt: a PDF /Subtype name in a test fixture.
                subtype: "Square".to_owned(),
                locked,
            },
            outline: Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 30.0)),
            oriented: None,
        });
        state
    }

    /// **A markup drags and a ce dimension does not.**
    #[test]
    fn only_ordinary_markup_is_this_modules_business() {
        assert!(eligible(&selection(AnnotKind::Markup, false)).is_some());
        assert!(eligible(&selection(AnnotKind::CeDimension, false)).is_none());
    }

    /// **A locked annotation does not drag, and draws no ghost either.**
    #[test]
    fn a_locked_annotation_offers_no_ghost() {
        assert!(eligible(&selection(AnnotKind::Markup, true)).is_none());
    }

    /// **A ghost tracks the pointer, and the committing frame draws none.**
    #[test]
    fn the_ghost_is_the_outline_translated_and_stops_on_commit() {
        let state = selection(AnnotKind::Markup, false);
        let mut actions = Vec::new();
        let moving = drag(
            &Frame {
                delta: egui::vec2(5.0, -7.0),
                phase: Phase::InFlight,
            },
            // No page is consulted on a non-committing frame, which is what
            // lets this be tested without one.
            None,
            &state,
            &mut actions,
        )
        .expect("a ghost");
        assert_eq!(moving.min, egui::pos2(15.0, 13.0));
        assert!(
            actions.is_empty(),
            "a frame that is not the release must raise nothing"
        );
    }

    /// **Nothing is raised for a selection this module does not own.**
    #[test]
    fn a_dimension_release_raises_nothing() {
        let mut actions = Vec::new();
        let ghost = drag(
            &Frame {
                delta: egui::vec2(5.0, -7.0),
                phase: Phase::Complete,
            },
            None,
            &selection(AnnotKind::CeDimension, false),
            &mut actions,
        );
        assert!(ghost.is_none());
        assert!(actions.is_empty());
    }
}
